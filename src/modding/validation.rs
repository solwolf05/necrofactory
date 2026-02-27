use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use bevy::prelude::*;
use semver::{Version, VersionReq};

use crate::modding::{Id, ModInfo, ModLoadState, ModRegistry, PathSegment};

pub fn validate_mods(
    mut next_state: ResMut<NextState<ModLoadState>>,
    mut mods: ResMut<ModRegistry>,
) {
    #[cfg(feature = "no_disable")]
    info!("NODISABLE is true");

    let instant = Instant::now();

    if let Err(dep_errors) = validate_dependencies(&mods) {
        for error in dep_errors {
            #[cfg(not(feature = "no_disable"))]
            error
                .mods()
                .into_iter()
                .for_each(|segment| mods.disable_segment(segment));

            error!("Validation error: {}", error);
        }
    }

    if let Err(cycle_errors) = detect_cycles(&mods) {
        for error in cycle_errors {
            #[cfg(not(feature = "no_disable"))]
            error
                .mods()
                .into_iter()
                .for_each(|segment| mods.disable_segment(segment));

            error!("Validation error: {}", error);
        }
    }

    match topological_sort(&mods) {
        Ok(order) => {
            mods.load_order = order;

            let elapsed = instant.elapsed();

            #[cfg(feature = "time")]
            info!("Mod validation complete ({}ms)", elapsed.as_millis_f32());

            #[cfg(not(feature = "time"))]
            info!("Mod validation complete");

            next_state.set(ModLoadState::Register);
        }
        Err(errors) => {
            for error in errors {
                #[cfg(not(feature = "no_disable"))]
                error
                    .mods()
                    .into_iter()
                    .for_each(|segment| mods.disable_segment(segment));

                error!("Validation error: {}", error);
            }
        }
    }
}

/// Error that can occur during mod validation
#[derive(Debug)]
pub enum ModValidationError {
    /// A version constraint is not satisfied
    VersionMismatch {
        mod_id: PathSegment,
        dependency_id: PathSegment,
        required: String,
        found: Option<String>,
    },
    /// Circular dependency detected
    CircularDependency { cycle: Vec<PathSegment> },
}

impl ModValidationError {
    #[allow(dead_code)]
    /// Returns the IDs of the mods involved in the error
    pub fn mods(&self) -> Vec<&PathSegment> {
        match self {
            ModValidationError::VersionMismatch { mod_id, .. } => vec![mod_id],
            ModValidationError::CircularDependency { cycle } => cycle.iter().collect(),
        }
    }
}

impl std::fmt::Display for ModValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModValidationError::VersionMismatch {
                mod_id,
                dependency_id: dependency,
                required,
                found: None,
            } => {
                write!(
                    f,
                    "mod '{}' requires '{}' version '{}', but it is not present",
                    mod_id, dependency, required,
                )
            }
            ModValidationError::VersionMismatch {
                mod_id,
                dependency_id: dependency,
                required,
                found: Some(found),
            } => {
                write!(
                    f,
                    "mod '{}' requires '{}' version '{}', but found version '{}'",
                    mod_id, dependency, required, found,
                )
            }
            ModValidationError::CircularDependency { cycle } => {
                let cycle_str = cycle
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                write!(f, "circular dependency detected: {}", cycle_str)
            }
        }
    }
}

impl std::error::Error for ModValidationError {}

/// Validate that all dependencies exist and version constraints are satisfied
fn validate_dependencies(registry: &ModRegistry) -> Result<(), Vec<ModValidationError>> {
    let mut errors = Vec::new();

    for (mod_id, mod_info) in registry.iter_enabled() {
        validate_required_dependencies(registry, &mut errors, mod_id, mod_info);
        validate_optional_dependencies(registry, &mut errors, mod_id, mod_info);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_required_dependencies(
    registry: &ModRegistry,
    errors: &mut Vec<ModValidationError>,
    mod_id: &PathSegment,
    mod_info: &ModInfo,
) {
    for (dep_id, version_req) in mod_info.dependencies() {
        let Some(dep_info) = registry.get_by_segment(&dep_id) else {
            errors.push(ModValidationError::VersionMismatch {
                mod_id: mod_id.clone(),
                dependency_id: dep_id.clone(),
                required: version_req.clone(),
                found: None,
            });
            continue;
        };

        if !dep_info.enabled() {
            errors.push(ModValidationError::VersionMismatch {
                mod_id: mod_id.clone(),
                dependency_id: dep_id.clone(),
                required: version_req.clone(),
                found: None,
            });
            continue;
        }

        if let (Ok(req), Ok(ver)) = (
            VersionReq::parse(version_req),
            Version::parse(dep_info.version()),
        ) {
            if !req.matches(&ver) {
                errors.push(ModValidationError::VersionMismatch {
                    mod_id: mod_id.clone(),
                    dependency_id: dep_id.clone().into(),
                    required: version_req.clone(),
                    found: Some(dep_info.version().to_owned()),
                });
            }
        }
    }
}

fn validate_optional_dependencies(
    registry: &ModRegistry,
    errors: &mut Vec<ModValidationError>,
    mod_id: &PathSegment,
    mod_info: &ModInfo,
) {
    for (dep_id, version_req) in mod_info.optional_dependencies() {
        let Some(dep_info) = registry.get_by_segment(&dep_id) else {
            continue;
        };

        if !dep_info.enabled() {
            continue;
        }

        if let (Ok(req), Ok(ver)) = (
            VersionReq::parse(version_req),
            Version::parse(dep_info.version()),
        ) {
            if !req.matches(&ver) {
                errors.push(ModValidationError::VersionMismatch {
                    mod_id: mod_id.clone(),
                    dependency_id: dep_id.clone().into(),
                    required: version_req.clone(),
                    found: Some(dep_info.version().to_owned()),
                });
            }
        }
    }
}

/// Detect circular dependencies using DFS
fn detect_cycles(registry: &ModRegistry) -> Result<(), Vec<ModValidationError>> {
    let mut errors = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();

    fn dfs(
        id: Id<ModInfo>,
        registry: &ModRegistry,
        visited: &mut HashSet<Id<ModInfo>>,
        rec_stack: &mut HashSet<Id<ModInfo>>,
        path: &mut Vec<Id<ModInfo>>,
        errors: &mut Vec<ModValidationError>,
    ) -> bool {
        visited.insert(id);
        rec_stack.insert(id);
        path.push(id);

        if let Some(mod_info) = registry.get(id) {
            for (dep_segment, _) in mod_info.dependencies() {
                let Some(dep_id) = registry.lookup(dep_segment) else {
                    #[cfg(feature = "no_disable")]
                    continue;

                    #[cfg(not(feature = "no_disable"))]
                    unreachable!("should have already been validated");
                };

                let dep_info = registry.get(dep_id).unwrap();
                if !dep_info.enabled() {
                    continue;
                }

                if !visited.contains(&dep_id) {
                    if dfs(dep_id, registry, visited, rec_stack, path, errors) {
                        return true;
                    }
                } else if rec_stack.contains(&dep_id) {
                    let cycle_start = path.iter().position(|&x| x == dep_id).unwrap();
                    let cycle: Vec<PathSegment> = path[cycle_start..]
                        .into_iter()
                        .map(|&id| registry.resolve(id).unwrap().clone())
                        .collect();
                    errors.push(ModValidationError::CircularDependency { cycle });
                    return true;
                }
            }

            for (dep_segment, _) in mod_info.optional_dependencies() {
                let Some(dep_id) = registry.lookup(dep_segment) else {
                    continue;
                };

                let dep_info = registry.get(dep_id).unwrap();
                if !dep_info.enabled() {
                    continue;
                }

                if !visited.contains(&dep_id) {
                    if dfs(dep_id, registry, visited, rec_stack, path, errors) {
                        return true;
                    }
                } else if rec_stack.contains(&dep_id) {
                    let cycle_start = path.iter().position(|&x| x == dep_id).unwrap();
                    let cycle: Vec<PathSegment> = path[cycle_start..]
                        .into_iter()
                        .map(|&id| registry.resolve(id).unwrap().clone())
                        .collect();
                    errors.push(ModValidationError::CircularDependency { cycle });
                    return true;
                }
            }
        }

        path.pop();
        rec_stack.remove(&id);
        false
    }

    for (id, ..) in registry.iter_enabled_with_id() {
        if !visited.contains(&id) {
            dfs(
                id,
                registry,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut errors,
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Perform topological sort using Kahn's algorithm
fn topological_sort(registry: &ModRegistry) -> Result<Vec<Id<ModInfo>>, Vec<ModValidationError>> {
    let mut in_degree: HashMap<Id<ModInfo>, usize> = HashMap::new();
    let mut adjacency: HashMap<Id<ModInfo>, Vec<Id<ModInfo>>> = HashMap::new();

    // Initialize in-degree and adjacency list
    for (id, ..) in registry.iter_enabled_with_id() {
        in_degree.insert(id, 0);
        adjacency.insert(id, Vec::new());
    }

    // Build the graph: for each dependency, add an edge from dependency to dependent
    for (mod_id, _, mod_info) in registry.iter_enabled_with_id() {
        for (dep_segment, _) in mod_info.dependencies() {
            if let Some(dep_id) = registry.lookup(dep_segment) {
                let dep_info = registry.get(dep_id).unwrap();
                if !dep_info.enabled() {
                    continue;
                }

                adjacency.entry(dep_id).and_modify(|deps| deps.push(mod_id));
                *in_degree.entry(mod_id).or_insert(0) += 1;
            }
        }

        for (dep_segment, _) in mod_info.optional_dependencies() {
            if let Some(dep_id) = registry.lookup(dep_segment) {
                let dep_info = registry.get(dep_id).unwrap();
                if !dep_info.enabled() {
                    continue;
                }

                adjacency.entry(dep_id).and_modify(|deps| deps.push(mod_id));
                *in_degree.entry(mod_id).or_insert(0) += 1;
            }
        }
    }

    // Kahn's algorithm: start with nodes that have zero in-degree
    let mut queue: Vec<Id<ModInfo>> = in_degree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut result: Vec<Id<ModInfo>> = Vec::new();

    // Sort the queue to ensure deterministic order
    queue.sort();

    while !queue.is_empty() {
        let current = queue.remove(0);
        result.push(current);

        // Decrease in-degree of neighbors
        if let Some(neighbors) = adjacency.get(&current) {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                        queue.sort(); // maintain deterministic order
                    }
                }
            }
        }
    }

    // If result doesn't contain all modules, there's a cycle (should be caught earlier, but be safe)
    if result.len() != registry.len_enabled() {
        return Err(vec![ModValidationError::CircularDependency {
            cycle: vec![],
        }]);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::modding::ModInfo;
    use crate::modding::ModMetadata;

    use super::*;

    /// Helper function to create a test mod registry
    fn create_registry(mods: Vec<(&str, Vec<(&str, &str)>, Vec<(&str, &str)>)>) -> ModRegistry {
        let mut registry = ModRegistry::default();
        for (id, deps, optional_deps) in mods {
            let mod_info = ModInfo {
                path: std::path::PathBuf::from(format!("/mod/{}", id)),
                metadata: ModMetadata {
                    id: PathSegment::new(id).unwrap(),
                    name: format!("{} Name", id),
                    version: "1.0.0".to_string(),
                    author: "Test Author".to_string(),
                    dependencies: deps
                        .into_iter()
                        .map(|(dep, ver)| (PathSegment::new(dep).unwrap(), ver.to_string()))
                        .collect(),
                    optional_dependencies: optional_deps
                        .into_iter()
                        .map(|(dep, ver)| (PathSegment::new(dep).unwrap(), ver.to_string()))
                        .collect(),
                },
                enabled: true,
            };
            registry.register(PathSegment::new(id).unwrap(), mod_info);
        }
        registry
    }

    /// Helper to extract the cycle from a CircularDependency error
    fn extract_cycle(errors: &[ModValidationError]) -> Option<Vec<String>> {
        for error in errors {
            if let ModValidationError::CircularDependency { cycle } = error {
                return Some(cycle.iter().map(|s| s.to_string()).collect());
            }
        }
        None
    }

    #[test]
    fn test_validate_dependencies_success() {
        // Create a registry with valid dependencies
        let mut registry = create_registry(vec![
            ("mod_a", vec![("mod_b", "^1.0")], vec![]),
            ("mod_b", vec![], vec![]),
        ]);

        // Register mod_b first with matching version
        let mod_b_info = registry
            .get_by_segment(&PathSegment::new("mod_b").unwrap())
            .unwrap()
            .clone();
        registry.register(PathSegment::new("mod_b").unwrap(), mod_b_info);

        // This should not produce any errors
        let result = validate_dependencies(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dependencies_missing_required() {
        let registry = create_registry(vec![("mod_a", vec![("mod_b", "^1.0")], vec![])]);

        let result = validate_dependencies(&registry);
        assert!(matches!(
            result,
            Err(errors) if errors.len() == 1 && matches!(&errors[0],
                ModValidationError::VersionMismatch { mod_id, dependency_id, required, found: None }
                if mod_id.to_string() == "mod_a" && dependency_id.to_string() == "mod_b" && required == "^1.0"
            )
        ));
    }

    // #[test]
    // fn test_validate_dependencies_version_mismatch() {
    //     let mut registry = create_registry(vec![
    //         ("mod_a", vec![("mod_b", "^2.0")], vec![]),
    //         ("mod_b", vec![], vec![]),
    //     ]);

    //     // Update mod_b to version 1.0.0 which doesn't satisfy ^2.0
    //     if let Some(mod_b_id) = registry.lookup(&PathSegment::new("mod_b").unwrap()) {
    //         if let Some(mod_b_info) = registry.get(mod_b_id).cloned() {
    //             let mut updated = mod_b_info.clone();
    //             updated.version = "1.0.0".to_string();
    //             registry.register(PathSegment::new("mod_b").unwrap(), updated);
    //         }
    //     }

    //     let result = validate_dependencies(&registry);
    //     assert!(matches!(
    //         result,
    //         Err(errors) if errors.len() == 1 && matches!(&errors[0],
    //             ModValidationError::VersionMismatch { mod_id, dependency_id, required, found: Some(found) }
    //             if mod_id.to_string() == "mod_a" && dependency_id.to_string() == "mod_b" && required == "^2.0" && found == "1.0.0"
    //         )
    //     ));
    // }

    #[test]
    fn test_validate_optional_dependencies_missing() {
        // Optional dependency that doesn't exist should not cause an error
        let registry = create_registry(vec![("mod_a", vec![], vec![("mod_b", "^1.0")])]);

        let result = validate_dependencies(&registry);
        assert!(result.is_ok());
    }

    // #[test]
    // fn test_validate_optional_dependencies_version_mismatch() {
    //     let mut registry = create_registry(vec![
    //         ("mod_a", vec![], vec![("mod_b", "^2.0")]),
    //         ("mod_b", vec![], vec![]),
    //     ]);

    //     // Update mod_b to version 1.0.0
    //     if let Some(mod_b_id) = registry.lookup(&PathSegment::new("mod_b").unwrap()) {
    //         if let Some(mod_b_info) = registry.get(mod_b_id).cloned() {
    //             let mut updated = mod_b_info.clone();
    //             updated.version = "1.0.0".to_string();
    //             registry.register(PathSegment::new("mod_b").unwrap(), updated);
    //         }
    //     }

    //     let result = validate_dependencies(&registry);
    //     // Version mismatch in optional deps should still be reported
    //     assert!(matches!(
    //         result,
    //         Err(errors) if errors.len() == 1 && matches!(&errors[0],
    //             ModValidationError::VersionMismatch { mod_id, dependency_id, required, found: Some(found) }
    //             if mod_id.to_string() == "mod_a" && required == "^2.0" && found == "1.0.0"
    //         )
    //     ));
    // }

    #[test]
    fn test_detect_cycles_no_cycle() {
        let registry = create_registry(vec![
            ("mod_a", vec![("mod_b", "^1.0")], vec![]),
            ("mod_b", vec![("mod_c", "^1.0")], vec![]),
            ("mod_c", vec![], vec![]),
        ]);

        let result = detect_cycles(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_cycles_simple_cycle() {
        // A -> B -> C -> A
        let registry = create_registry(vec![
            ("mod_a", vec![("mod_b", "^1.0")], vec![]),
            ("mod_b", vec![("mod_c", "^1.0")], vec![]),
            ("mod_c", vec![("mod_a", "^1.0")], vec![]),
        ]);

        let result = detect_cycles(&registry);
        assert!(matches!(
            result,
            Err(errors) if matches!(extract_cycle(&errors), Some(cycle) if cycle.len() == 3)
        ));
    }

    #[test]
    fn test_detect_cycles_self_reference() {
        let registry = create_registry(vec![("mod_a", vec![("mod_a", "^1.0")], vec![])]);

        let result = detect_cycles(&registry);
        assert!(matches!(
            result,
            Err(errors) if matches!(extract_cycle(&errors), Some(cycle) if cycle.len() == 1)
        ));
    }

    #[test]
    fn test_topological_sort_simple() {
        let registry = create_registry(vec![
            ("mod_c", vec![("mod_a", "^1.0")], vec![]),
            ("mod_b", vec![("mod_a", "^1.0")], vec![]),
            ("mod_a", vec![], vec![]),
        ]);

        let result = topological_sort(&registry);
        assert!(result.is_ok());

        let order = result.unwrap();
        // mod_a should come before mod_b and mod_c
        let mod_a_id = registry
            .lookup(&PathSegment::new("mod_a").unwrap())
            .unwrap();
        let mod_b_id = registry
            .lookup(&PathSegment::new("mod_b").unwrap())
            .unwrap();
        let mod_c_id = registry
            .lookup(&PathSegment::new("mod_c").unwrap())
            .unwrap();

        let mod_a_pos = order.iter().position(|&id| id == mod_a_id).unwrap();
        let mod_b_pos = order.iter().position(|&id| id == mod_b_id).unwrap();
        let mod_c_pos = order.iter().position(|&id| id == mod_c_id).unwrap();

        assert!(mod_a_pos < mod_b_pos);
        assert!(mod_a_pos < mod_c_pos);
    }

    #[test]
    fn test_topological_sort_with_cycle() {
        // A -> B -> C -> A
        let registry = create_registry(vec![
            ("mod_a", vec![("mod_b", "^1.0")], vec![]),
            ("mod_b", vec![("mod_c", "^1.0")], vec![]),
            ("mod_c", vec![("mod_a", "^1.0")], vec![]),
        ]);

        let result = topological_sort(&registry);
        assert!(matches!(
            result,
            Err(errors) if matches!(errors.first(), Some(ModValidationError::CircularDependency { .. }))
        ));
    }

    // #[test]
    // fn test_validate_mods_success() {
    //     let mut registry = create_registry(vec![
    //         ("mod_a", vec![("mod_b", "^1.0")], vec![]),
    //         ("mod_b", vec![], vec![]),
    //     ]);

    //     // Update mod_b to have a matching version
    //     if let Some(mod_b_id) = registry.lookup(&PathSegment::new("mod_b").unwrap()) {
    //         if let Some(mod_b_info) = registry.get(mod_b_id).cloned() {
    //             let mut updated = mod_b_info.clone();
    //             updated.version = "1.0.0".to_string();
    //             registry.register(PathSegment::new("mod_b").unwrap(), updated);
    //         }
    //     }

    //     // We can't actually test validate_mods without a full bevy app,
    //     // but we can test the individual functions
    //     assert!(detect_cycles(&registry).is_ok());
    //     assert!(validate_dependencies(&registry).is_ok());
    //     assert!(topological_sort(&registry).is_ok());
    // }

    #[test]
    fn test_validation_error_display_version_mismatch() {
        let error = ModValidationError::VersionMismatch {
            mod_id: PathSegment::new("mod_a").unwrap(),
            dependency_id: PathSegment::new("mod_b").unwrap(),
            required: "^1.0".to_string(),
            found: None,
        };

        let expected = "Mod 'mod_a' requires 'mod_b' version '^1.0', but it is not present";
        assert_eq!(error.to_string(), expected);

        let error_with_found = ModValidationError::VersionMismatch {
            mod_id: PathSegment::new("mod_a").unwrap(),
            dependency_id: PathSegment::new("mod_b").unwrap(),
            required: "^2.0".to_string(),
            found: Some("1.0.0".to_string()),
        };

        let expected = "Mod 'mod_a' requires 'mod_b' version '^2.0', but found version '1.0.0'";
        assert_eq!(error_with_found.to_string(), expected);
    }

    #[test]
    fn test_validation_error_display_circular_dependency() {
        let error = ModValidationError::CircularDependency {
            cycle: vec![
                PathSegment::new("a").unwrap(),
                PathSegment::new("b").unwrap(),
                PathSegment::new("c").unwrap(),
            ],
        };

        let expected = "Circular dependency detected: a -> b -> c";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_complex_dependency_graph() {
        // A -> B, C
        // B -> D
        // C -> D, E
        // D -> E
        // E -> (none)
        let registry = create_registry(vec![
            ("mod_a", vec![("mod_b", "^1.0"), ("mod_c", "^1.0")], vec![]),
            ("mod_b", vec![("mod_d", "^1.0")], vec![]),
            ("mod_c", vec![("mod_d", "^1.0"), ("mod_e", "^1.0")], vec![]),
            ("mod_d", vec![("mod_e", "^1.0")], vec![]),
            ("mod_e", vec![], vec![]),
        ]);

        assert!(detect_cycles(&registry).is_ok());
        assert!(validate_dependencies(&registry).is_ok());

        let order = topological_sort(&registry).unwrap();
        let e_id = registry
            .lookup(&PathSegment::new("mod_e").unwrap())
            .unwrap();
        let d_id = registry
            .lookup(&PathSegment::new("mod_d").unwrap())
            .unwrap();
        let c_id = registry
            .lookup(&PathSegment::new("mod_c").unwrap())
            .unwrap();
        let b_id = registry
            .lookup(&PathSegment::new("mod_b").unwrap())
            .unwrap();
        let a_id = registry
            .lookup(&PathSegment::new("mod_a").unwrap())
            .unwrap();

        let e_pos = order.iter().position(|&id| id == e_id).unwrap();
        let d_pos = order.iter().position(|&id| id == d_id).unwrap();
        let c_pos = order.iter().position(|&id| id == c_id).unwrap();
        let b_pos = order.iter().position(|&id| id == b_id).unwrap();
        let a_pos = order.iter().position(|&id| id == a_id).unwrap();

        // E should come first (no dependencies)
        assert!(e_pos < d_pos);
        assert!(e_pos < c_pos);
        assert!(e_pos < b_pos);
        assert!(e_pos < a_pos);

        // D should come before C and A
        assert!(d_pos < c_pos);
        assert!(d_pos < a_pos);

        // B and C should come before A
        assert!(b_pos < a_pos);
        assert!(c_pos < a_pos);
    }

    #[test]
    fn test_version_requirements() {
        // Test various version requirements
        let tests = vec![
            ("1.0.0", "^1.0", true), // Major version match
            ("1.5.0", "^1.0", true),
            ("2.0.0", "^1.0", false), // Different major
            ("1.0.0", "~1.0", true),  // Minor version match
            ("1.0.5", "~1.0", true),
            ("1.1.0", "~1.0", false),   // Different minor
            ("1.0.0", ">=1.0.0", true), // Greater or equal
            ("2.0.0", ">=1.0.0", true),
            ("0.9.0", ">=1.0.0", false),
            ("1.0.0", "<2.0.0", true), // Less than
            ("1.5.0", "<2.0.0", true),
            ("2.0.0", "<2.0.0", false),
        ];

        for (version, req, should_match) in tests {
            let mod_version = Version::parse(version).unwrap();
            let version_req = VersionReq::parse(req).unwrap();
            assert_eq!(
                version_req.matches(&mod_version),
                should_match,
                "Version {} with requirement {} should {} match",
                version,
                req,
                if should_match { "match" } else { "not match" }
            );
        }
    }
}
