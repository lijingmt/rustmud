// core/program.rs - 程序加载系统
// 对应 Pike 的 program 类型和动态加载

use crate::core::{MudError, Result};
use std::path::Path;
use std::collections::HashMap;

/// 程序类型 (对应 Pike 的 program)
#[derive(Debug, Clone)]
pub struct Program {
    pub path: String,
    pub source: String,
    pub compiled: bool,
    pub functions: Vec<String>,
}

/// 程序管理器 (对应 master()->cast_to_program)
pub struct ProgramManager {
    programs: HashMap<String, Program>,
    search_paths: Vec<String>,
}

impl ProgramManager {
    pub fn new() -> Self {
        Self {
            programs: HashMap::new(),
            search_paths: vec![
                "/pikenv/".to_string(),
                "/gamenv/".to_string(),
                "/gamenv/cmds/".to_string(),
                "/gamenv/single/daemons/".to_string(),
            ],
        }
    }

    /// 添加搜索路径 (对应 Pike 的 add_program_path)
    pub fn add_program_path(&mut self, path: String) {
        self.search_paths.push(path);
    }

    /// 对应 Pike 的 cast_to_program()
    pub fn cast_to_program(&mut self, pname: &str) -> Result<Program> {
        // 首先检查缓存
        if let Some(prog) = self.programs.get(pname) {
            return Ok(prog.clone());
        }

        // 搜索文件
        let full_path = self.find_program_file(pname)?;

        // 读取源代码
        let source = std::fs::read_to_string(&full_path)
            .map_err(|e| MudError::ObjectNotFound(format!("{}: {}", full_path, e)))?;

        let program = Program {
            path: full_path.clone(),
            source: source.clone(),
            compiled: false,
            functions: Vec::new(),
        };

        // 缓存程序
        self.programs.insert(full_path, program.clone());
        self.programs.insert(pname.to_string(), program.clone());

        Ok(program)
    }

    /// 查找程序文件
    fn find_program_file(&self, pname: &str) -> Result<String> {
        // 如果是绝对路径
        if pname.starts_with('/') {
            if Path::new(pname).exists() {
                return Ok(pname.to_string());
            }
            return Err(MudError::ObjectNotFound(pname.to_string()));
        }

        // 在搜索路径中查找
        for search_path in &self.search_paths {
            let full_path = format!("{}{}.rs", search_path, pname.trim_end_matches(".pike"));
            if Path::new(&full_path).exists() {
                return Ok(full_path);
            }
            // 也检查 .rs 扩展名
            let full_path_rs = format!("{}{}.rs", search_path, pname.trim_end_matches(".rs"));
            if Path::new(&full_path_rs).exists() {
                return Ok(full_path_rs);
            }
        }

        Err(MudError::ObjectNotFound(format!("Program not found: {}", pname)))
    }

    /// 清除缓存 (对应 Pike 的程序重新加载)
    pub fn clear_cache(&mut self) {
        self.programs.clear();
    }

    /// 清除特定程序缓存
    pub fn clear_program_cache(&mut self, pname: &str) {
        self.programs.remove(pname);
    }
}

impl Default for ProgramManager {
    fn default() -> Self {
        Self::new()
    }
}
