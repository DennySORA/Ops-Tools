//! 模板模組 - 將所有模板集中管理
//!
//! 包含 4 個主要模板與共用區塊，以及 YAML 生成 prompt

mod frontend_section;
mod state_requirement;
mod template_01;
mod template_02;
mod template_03;
mod template_04;
pub mod yaml_gen_prompt;

pub use frontend_section::FRONTEND_SECTION;
pub use state_requirement::STATE_REQUIREMENT_BLOCK;
pub use template_01::TEMPLATE_01;
pub use template_02::TEMPLATE_02_FIXED;
pub use template_03::TEMPLATE_03;
pub use template_04::TEMPLATE_04_FIXED;
#[allow(unused_imports)]
pub use yaml_gen_prompt::generate_yaml_prompt;
