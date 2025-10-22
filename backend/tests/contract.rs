mod common;

#[path = "contract/test_paper_import.rs"]
mod test_paper_import;
#[path = "contract/test_paper_import_edges.rs"]
mod test_paper_import_edges;
#[path = "contract/test_pdf_error_handling.rs"]
mod test_pdf_error_handling;

#[path = "contract/test_paper_list.rs"]
mod test_paper_list;

#[path = "contract/test_paper_detail.rs"]
mod test_paper_detail;

#[path = "contract/test_translation.rs"]
mod test_translation;

#[path = "contract/test_paper_process.rs"]
mod test_paper_process;
#[path = "contract/test_paper_status.rs"]
mod test_paper_status;

#[path = "contract/test_paper_delete.rs"]
mod test_paper_delete;

#[path = "contract/test_chunk_retry.rs"]
mod test_chunk_retry;
#[path = "contract/test_chunk_retry_success.rs"]
mod test_chunk_retry_success;

#[path = "contract/test_term_detail.rs"]
mod test_term_detail;

#[path = "contract/test_occurrences.rs"]
mod test_occurrences;

#[path = "contract/test_term_list.rs"]
mod test_term_list;
#[path = "contract/test_term_list_sorting.rs"]
mod test_term_list_sorting;

#[path = "contract/test_term_create.rs"]
mod test_term_create;

#[path = "contract/test_term_update.rs"]
mod test_term_update;

#[path = "contract/test_term_delete.rs"]
mod test_term_delete;

#[path = "contract/test_term_merge.rs"]
mod test_term_merge;
#[path = "contract/test_term_duplicates.rs"]
mod test_term_duplicates;

#[path = "contract/test_term_define.rs"]
mod test_term_define;
#[path = "contract/test_term_define_not_found.rs"]
mod test_term_define_not_found;

#[path = "contract/test_term_merge_validation.rs"]
mod test_term_merge_validation;

#[path = "contract/test_term_list_validation.rs"]
mod test_term_list_validation;

#[path = "contract/test_term_search_normalization.rs"]
mod test_term_search_normalization;
#[path = "contract/test_translation_order.rs"]
mod test_translation_order;
