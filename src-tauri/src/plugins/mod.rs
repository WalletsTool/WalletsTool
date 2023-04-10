use tauri::{Invoke, plugin::Plugin, Runtime};

pub mod fs_extra;

/// Tauri plugin.
pub struct ToolsExtra<R: Runtime> {
  invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Default for ToolsExtra<R> {
  fn default() -> Self {
      Self {
          invoke_handler: Box::new(tauri::generate_handler![
            // fs
            fs_extra::exists,
            fs_extra::open_file,
          ]),
      }
  }
}

impl<R: Runtime> Plugin<R> for ToolsExtra<R> {
  fn name(&self) -> &'static str {
      "extra_tools"
  }

  fn extend_api(&mut self, message: Invoke<R>) {
      (self.invoke_handler)(message)
  }
}
