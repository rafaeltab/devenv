//! Tester test modules.
//!
//! These tests verify the behavior of all tester types defined in the
//! testing infrastructure.

mod assertion_tests;
mod client_cleanup_tests;
mod client_creation_tests;
mod client_query_tests;
mod cmd_tester_tests;
mod color_tests;
mod command_builder_tests;
mod command_tests;
mod input_tests;
mod lifecycle_tests;
mod pty_tester_tests;
mod screen_tests;
mod terminal_sequence_tests;
mod tester_integration_tests;
mod text_finding_tests;
mod tmux_client_cmd_tests;
mod tmux_client_pty_tests;
mod tmux_full_client_tests;
mod wait_settle_tests;

// Re-export test programs module
#[path = "../test_programs/mod.rs"]
mod test_programs;
