use crate::process::ProcessInfo;
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct ProcessTree {
    pub process: ProcessInfo,
    pub children: Vec<ProcessTree>,
    pub level: usize,
}

impl ProcessTree {
    pub fn build_tree(processes: &[ProcessInfo]) -> Vec<ProcessTree> {
        debug!("Building process tree from {} processes", processes.len());
        let mut process_map: HashMap<u32, ProcessInfo> = HashMap::new();
        let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
        let mut roots = Vec::new();

        // Build maps for quick lookup
        for process in processes {
            process_map.insert(process.pid, process.clone());
            children_map.entry(process.ppid).or_insert_with(Vec::new).push(process.pid);
        }

        // Find root processes (processes with no parent or parent not in our list)
        for process in processes {
            if process.ppid == 0 || !process_map.contains_key(&process.ppid) {
                if let Some(tree) = Self::build_subtree(process.pid, &process_map, &children_map, 0) {
                    roots.push(tree);
                }
            }
        }

        // Sort roots by PID
        roots.sort_by_key(|tree| tree.process.pid);
        info!("Built process tree with {} root processes", roots.len());
        roots
    }

    fn build_subtree(
        pid: u32,
        process_map: &HashMap<u32, ProcessInfo>,
        children_map: &HashMap<u32, Vec<u32>>,
        level: usize,
    ) -> Option<ProcessTree> {
        if let Some(process) = process_map.get(&pid) {
            let mut children = Vec::new();
            
            if let Some(child_pids) = children_map.get(&pid) {
                for &child_pid in child_pids {
                    if let Some(child_tree) = Self::build_subtree(child_pid, process_map, children_map, level + 1) {
                        children.push(child_tree);
                    }
                }
            }

            // Sort children by PID
            children.sort_by_key(|tree| tree.process.pid);

            Some(ProcessTree {
                process: process.clone(),
                children,
                level,
            })
        } else {
            None
        }
    }

    pub fn flatten(&self) -> Vec<(ProcessInfo, usize)> {
        let mut result = Vec::new();
        self.flatten_recursive(&mut result);
        result
    }

    fn flatten_recursive(&self, result: &mut Vec<(ProcessInfo, usize)>) {
        result.push((self.process.clone(), self.level));
        for child in &self.children {
            child.flatten_recursive(result);
        }
    }

    pub fn format_tree_line(&self) -> String {
        let indent = "  ".repeat(self.level);
        let prefix = if self.level > 0 {
            "├─ "
        } else {
            ""
        };
        
        format!("{}{}{}", indent, prefix, self.process.name)
    }

    pub fn count_processes(&self) -> usize {
        1 + self.children.iter().map(|child| child.count_processes()).sum::<usize>()
    }

    pub fn find_process(&self, pid: u32) -> Option<&ProcessTree> {
        if self.process.pid == pid {
            Some(self)
        } else {
            self.children.iter().find_map(|child| child.find_process(pid))
        }
    }

    pub fn get_all_descendants(&self) -> Vec<u32> {
        let mut descendants = Vec::new();
        for child in &self.children {
            descendants.push(child.process.pid);
            descendants.extend(child.get_all_descendants());
        }
        descendants
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessInfo;
    use std::time::Duration;

    fn create_test_process(pid: u32, ppid: u32, name: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid,
            name: name.to_string(),
            command: format!("/usr/bin/{}", name),
            user: "test".to_string(),
            cpu_usage: 0.0,
            memory_usage: 1024,
            memory_percent: 0.1,
            status: "Running".to_string(),
            start_time: 0,
            running_time: Duration::from_secs(100),
            uid: 1000,
            gid: 1000,
            threads: 1,
            priority: 20,
            nice: 0,
            network_connections: None,
            is_container: false,
            container_id: None,
            cgroup_memory_limit: None,
            gpu_memory: None,
        }
    }

    #[test]
    fn test_build_simple_tree() {
        let processes = vec![
            create_test_process(1, 0, "init"),
            create_test_process(2, 1, "kthreadd"),
            create_test_process(3, 2, "ksoftirqd"),
        ];

        let trees = ProcessTree::build_tree(&processes);
        assert_eq!(trees.len(), 1);
        assert_eq!(trees[0].process.pid, 1);
        assert_eq!(trees[0].children.len(), 1);
        assert_eq!(trees[0].children[0].process.pid, 2);
        assert_eq!(trees[0].children[0].children.len(), 1);
        assert_eq!(trees[0].children[0].children[0].process.pid, 3);
    }

    #[test]
    fn test_flatten_tree() {
        let processes = vec![
            create_test_process(1, 0, "init"),
            create_test_process(2, 1, "child1"),
            create_test_process(3, 1, "child2"),
        ];

        let trees = ProcessTree::build_tree(&processes);
        let flattened = trees[0].flatten();
        
        assert_eq!(flattened.len(), 3);
        assert_eq!(flattened[0].0.pid, 1);
        assert_eq!(flattened[0].1, 0); // level 0
        assert_eq!(flattened[1].0.pid, 2);
        assert_eq!(flattened[1].1, 1); // level 1
        assert_eq!(flattened[2].0.pid, 3);
        assert_eq!(flattened[2].1, 1); // level 1
    }
}