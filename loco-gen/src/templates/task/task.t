{% set file_name = name |  snake_case -%}
{% set module_name = file_name | pascal_case -%}
to: "src/tasks/{{file_name}}.rs"
skip_exists: true
message: "A Task `{{module_name}}`{% if is_git_task %}(git task){% endif %} was added successfully. Run with `cargo run task {{name}}`."
injections:
- into: "src/tasks/mod.rs"
  append: true
  content: "pub mod {{ file_name }};"
- into: src/app.rs
  before: "// tasks-inject"
  content: "        tasks.register(tasks::{{file_name}}::{{module_name}});"
{% if is_git_task %}
- into: Cargo.toml
  after: "[dependencies]"
  content: '\n{{file_name}} = { path = "./tasks/{{file_name}}" }'
- into: "tasks/{{file_name}}/Cargo.toml"
  after: "[dependencies]"
  content: '\npkg_root = { package = "{{pkg_name}}", path = "../../../" }'
{% endif %}
---
{% if is_git_task %}
use {{file_name}}::*;
{% else %}
use loco_rs::prelude::*;

pub struct {{module_name}};
#[async_trait]
impl Task for {{module_name}} {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "{{name}}".to_string(),
            detail: "Task generator".to_string(),
        }
    }
    async fn run(&self, _app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        println!("Task {{module_name}} generated");
        Ok(())
    }
}
{% endif %}
