pub mod import;
pub mod list;
pub mod detail;
pub mod translation;
pub mod process;
pub mod status;
pub mod file;

pub use import::import_paper;
pub use list::list_papers;
pub use detail::get_paper;
pub use translation::get_translation;
pub use process::process_paper;
pub use status::get_paper_status;
pub use file::get_paper_file;
