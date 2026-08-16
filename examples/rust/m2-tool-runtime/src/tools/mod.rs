//! Pi 风格七工具。

mod bash;
mod edit;
pub mod find;
mod grep;
mod ls;
mod read;
mod write;

pub use bash::BashTool;
pub use edit::EditTool;
pub use find::FindTool;
pub use grep::GrepTool;
pub use ls::LsTool;
pub use read::ReadTool;
pub use write::WriteTool;

use crate::registry::ToolRegistry;
use crate::workspace::Workspace;

/// 按固定顺序注册七工具；进程工具默认由调用者决定是否启用。
pub fn build_default_registry(
    workspace: Workspace,
    enable_bash: bool,
    bash_allowlist: Option<Vec<String>>,
) -> ToolRegistry {
    let mut registry = ToolRegistry::default();
    registry.register(ReadTool::new(workspace.clone()));
    registry.register(WriteTool::new(workspace.clone()));
    registry.register(EditTool::new(workspace.clone()));
    registry.register(GrepTool::new(workspace.clone()));
    registry.register(FindTool::new(workspace.clone()));
    registry.register(LsTool::new(workspace.clone()));
    registry.register(BashTool::new(workspace, enable_bash, bash_allowlist));
    registry
}
