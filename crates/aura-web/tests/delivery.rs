use aura_web::{WebError, WebService};

const PAGE: &str = r#"{"text":"PRIVATE_PAGE_039","externalContent":true}"#;

#[test]
fn later_reasoning_keeps_the_model_response_private_after_tool_results_expire() {
    struct Clock(std::sync::atomic::AtomicU64);
    impl aura_web::Clock for Clock {
        fn now(&self) -> std::time::SystemTime {
            std::time::UNIX_EPOCH
                + std::time::Duration::from_secs(self.0.load(std::sync::atomic::Ordering::SeqCst))
        }
    }
    let clock = std::sync::Arc::new(Clock(std::sync::atomic::AtomicU64::new(0)));
    let service = WebService::new(
        std::sync::Arc::new(aura_web::transport::ReqwestTransport),
        clock.clone(),
    );
    let context = service.begin_turn("conversation", "turn");
    service
        .retain_tool_result(&context, "thread", "web_fetch", PAGE.into())
        .unwrap();
    clock.0.store(800, std::sync::atomic::Ordering::SeqCst);
    let reasoning = service.retain_private_reasoning(&context,"thread",r#"{"type":"reasoning","id":"rs_private","summary":[],"encrypted_content":"PRIVATE_OPAQUE_BLOB"}"#.into()).unwrap();
    clock.0.store(901, std::sync::atomic::Ordering::SeqCst);
    assert!(
        service
            .resolve_private_reasoning("conversation", "thread", &reasoning)
            .unwrap()
            .is_some()
    );
    assert!(
        service.has_private_context(&context),
        "Fresh derived reasoning is still private after the older tool result expires"
    );
}

#[test]
fn private_arguments_and_results_share_budget_but_never_share_reference_authority() {
    let service = WebService::live();
    let context = service.begin_turn("conversation", "turn");
    let args = service
        .retain_tool_arguments(
            &context,
            "thread",
            "web_fetch",
            r#"{"url":"https://private.example"}"#.into(),
        )
        .unwrap();
    let result = service
        .retain_tool_result(&context, "thread", "web_fetch", PAGE.into())
        .unwrap();
    assert_eq!(
        service.resolve_tool_result("conversation", "thread", "web_fetch", &args),
        Err(WebError::InvalidInput)
    );
    assert_eq!(
        service.resolve_tool_arguments("conversation", "thread", "web_fetch", &result),
        Err(WebError::InvalidInput)
    );
    for _ in 0..15 {
        service
            .retain_tool_arguments(&context, "thread", "web_fetch", "{}".into())
            .unwrap();
        service
            .retain_tool_result(&context, "thread", "web_fetch", PAGE.into())
            .unwrap();
    }
    assert_eq!(
        service.retain_tool_arguments(&context, "thread", "web_fetch", "{}".into()),
        Err(WebError::LimitExceeded)
    );
    assert_eq!(
        service.retain_tool_result(&context, "thread", "web_fetch", PAGE.into()),
        Err(WebError::LimitExceeded)
    );
    service.cancel_turn(&context);
    assert!(
        service
            .resolve_tool_arguments("conversation", "thread", "web_fetch", &args)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        service.retain_tool_arguments(&context, "thread", "web_fetch", "{}".into()),
        Err(WebError::Cancelled)
    );
}

#[test]
fn issued_reference_delivers_the_literal_only_with_its_issuing_scope() {
    let service = WebService::live();
    let context = service.begin_turn("conversation-A", "turn-A");
    let handle = service
        .retain_tool_result(&context, "thread-A", "web_fetch", PAGE.into())
        .unwrap();
    assert_eq!(handle.len(), 64);
    assert!(handle.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert!(!handle.contains("PRIVATE_PAGE_039"));
    for _ in 0..2 {
        let delivered = service
            .resolve_tool_result("conversation-A", "thread-A", "web_fetch", &handle)
            .unwrap()
            .unwrap();
        assert_eq!(delivered.as_str(), PAGE);
    }
}

#[test]
fn rejected_new_turn_releases_cancelled_results_from_the_global_budget() {
    let service = WebService::live();
    let initial = service.begin_turn("old", "initial");
    for _ in 0..3 {
        service
            .retain_tool_result(
                &initial,
                "old-thread",
                "web_fetch",
                "x".repeat(2 * 1024 * 1024),
            )
            .unwrap();
    }
    let rejected = service.begin_turn("old", &"t".repeat(9 * 1024 * 1024));
    assert_eq!(
        service.retain_tool_result(&rejected, "old-thread", "web_fetch", PAGE.into()),
        Err(WebError::LimitExceeded)
    );
    // 30 MiB is below the specified global 32 MiB, with room for tiny metadata.
    for index in 0..5 {
        let conversation = format!("other-{index}");
        let context = service.begin_turn(&conversation, "turn");
        for _ in 0..3 {
            assert!(
                service
                    .retain_tool_result(
                        &context,
                        &conversation,
                        "web_fetch",
                        "y".repeat(2 * 1024 * 1024)
                    )
                    .is_ok(),
                "cancelled private data must not occupy another conversation's budget"
            );
        }
    }
}

#[test]
fn a_reference_cannot_be_read_from_another_conversation_thread_or_tool() {
    let service = WebService::live();
    let context = service.begin_turn("conversation-A", "turn-A");
    service.begin_turn("conversation-B", "turn-B");
    let handle = service
        .retain_tool_result(&context, "thread-A", "web_fetch", PAGE.into())
        .unwrap();
    for (conversation, thread, tool) in [
        ("conversation-B", "thread-B", "web_fetch"),
        ("conversation-A", "thread-B", "web_fetch"),
        ("conversation-A", "thread-A", "web_search"),
    ] {
        assert!(matches!(
            service.resolve_tool_result(conversation, thread, tool, &handle),
            Err(WebError::InvalidInput)
        ));
    }
    assert!(
        service
            .resolve_tool_result("conversation-A", "thread-A", "web_fetch", &"0".repeat(64))
            .unwrap()
            .is_none()
    );
}

#[test]
fn disabling_cancelling_replacing_and_closing_never_revive_old_results() {
    for action in ["disable", "cancel", "next-turn", "close"] {
        let service = WebService::live();
        let old = service.begin_turn("conversation", "turn-A");
        let handle = service
            .retain_tool_result(&old, "thread", "web_fetch", PAGE.into())
            .unwrap();
        match action {
            "disable" => {
                service.set_enabled(false);
                service.set_enabled(true);
            }
            "cancel" => service.cancel_turn(&old),
            "next-turn" => {
                service.begin_turn("conversation", "turn-B");
            }
            "close" => service.cancel_conversation("conversation"),
            _ => unreachable!(),
        }
        assert!(
            service
                .resolve_tool_result("conversation", "thread", "web_fetch", &handle)
                .unwrap()
                .is_none(),
            "{action}"
        );
        assert_eq!(
            service.retain_tool_result(&old, "thread", "web_fetch", PAGE.into()),
            Err(WebError::Cancelled)
        );
        let new = service.begin_turn("conversation", "turn-C");
        let fresh = service
            .retain_tool_result(&new, "thread", "web_fetch", PAGE.into())
            .unwrap();
        service.cancel_turn(&old); // a late cancellation cannot clear the new session
        assert_eq!(
            service
                .resolve_tool_result("conversation", "thread", "web_fetch", &fresh)
                .unwrap()
                .unwrap()
                .as_str(),
            PAGE
        );
    }
}

#[test]
fn transient_results_obey_per_conversation_entries_and_byte_limits() {
    let service = WebService::live();
    let context = service.begin_turn("conversation", "turn");
    let first = service
        .retain_tool_result(&context, "thread", "web_fetch", PAGE.into())
        .unwrap();
    for _ in 1..32 {
        service
            .retain_tool_result(&context, "thread", "web_fetch", PAGE.into())
            .unwrap();
    }
    assert_eq!(
        service.retain_tool_result(&context, "thread", "web_fetch", PAGE.into()),
        Err(WebError::LimitExceeded)
    );
    assert_eq!(
        service
            .resolve_tool_result("conversation", "thread", "web_fetch", &first)
            .unwrap()
            .unwrap()
            .as_str(),
        PAGE
    );
    service.cancel_turn(&context);
    let context = service.begin_turn("conversation", "next-turn");
    for _ in 0..3 {
        service
            .retain_tool_result(&context, "thread", "web_fetch", "x".repeat(2 * 1024 * 1024))
            .unwrap();
    }
    assert_eq!(
        service.retain_tool_result(&context, "thread", "web_fetch", "x".repeat(2 * 1024 * 1024)),
        Err(WebError::LimitExceeded)
    );
}

#[test]
fn aggregate_private_results_cannot_exceed_the_shared_32_mib_budget() {
    let service = WebService::live();
    let mut contexts = Vec::new();
    for index in 0..5 {
        let conversation = format!("conversation-{index}");
        let context = service.begin_turn(&conversation, "turn");
        for _ in 0..3 {
            service
                .retain_tool_result(
                    &context,
                    &conversation,
                    "web_fetch",
                    "x".repeat(2 * 1024 * 1024),
                )
                .unwrap();
        }
        contexts.push(context);
    }
    let extra = service.begin_turn("extra", "turn");
    assert_eq!(
        service.retain_tool_result(&extra, "extra", "web_fetch", "x".repeat(2 * 1024 * 1024)),
        Err(WebError::LimitExceeded)
    );
    service.cancel_conversation("conversation-4");
    service
        .retain_tool_result(&extra, "extra", "web_fetch", "x".repeat(2 * 1024 * 1024))
        .unwrap();
}

#[test]
fn references_expire_at_fifteen_minutes_without_refetching() {
    struct Clock(std::sync::atomic::AtomicU64);
    impl aura_web::Clock for Clock {
        fn now(&self) -> std::time::SystemTime {
            std::time::UNIX_EPOCH
                + std::time::Duration::from_secs(self.0.load(std::sync::atomic::Ordering::SeqCst))
        }
    }
    let clock = std::sync::Arc::new(Clock(std::sync::atomic::AtomicU64::new(0)));
    let service = WebService::new(
        std::sync::Arc::new(aura_web::transport::ReqwestTransport),
        clock.clone(),
    );
    let context = service.begin_turn("conversation", "turn");
    let handle = service
        .retain_tool_result(&context, "thread", "web_fetch", PAGE.into())
        .unwrap();
    clock.0.store(899, std::sync::atomic::Ordering::SeqCst);
    assert!(
        service
            .resolve_tool_result("conversation", "thread", "web_fetch", &handle)
            .unwrap()
            .is_some()
    );
    clock.0.store(900, std::sync::atomic::Ordering::SeqCst);
    assert!(
        service
            .resolve_tool_result("conversation", "thread", "web_fetch", &handle)
            .unwrap()
            .is_none()
    );
}
