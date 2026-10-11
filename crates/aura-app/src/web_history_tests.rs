use super::WebHistory;
use aura_store::{
    Store,
    conversations::{ConversationMeta, ConversationsRepo},
};
use aura_web::WebSource;

fn normal(store: &Store, thread: &str) {
    ConversationsRepo::new(store)
        .upsert(&ConversationMeta {
            thread_id: thread.into(),
            conversation_uuid: format!("uuid-{thread}"),
            workspace_path: "workspace".into(),
            mode: "chat".into(),
            provider_id: "fixture".into(),
            granted_folders: vec![],
            extra_instructions: String::new(),
        })
        .unwrap();
}

fn source(id: &str, url: &str, title: &str) -> WebSource {
    WebSource {
        source_id: id.into(),
        title: title.into(),
        url: url.into(),
        snippet: "Uncited private page text must not be persisted here.".into(),
        published_at: None,
        retrieved_at: "2026-10-10T16:00:00Z".into(),
        kind: "pageContent".into(),
    }
}

#[test]
fn cited_metadata_is_isolated_in_a_real_database_after_reopening() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("history.sqlite");
    {
        let store = Store::open(&path).unwrap();
        normal(&store, "one");
        normal(&store, "two");
        let history = WebHistory::new(store).unwrap();
        history
            .record(
                "one",
                "turn",
                "answer",
                vec![source("W1", "https://one.example/report", "First report")],
            )
            .unwrap();
        history
            .record(
                "two",
                "turn",
                "answer",
                vec![source("W1", "https://two.example/report", "Second report")],
            )
            .unwrap();
    }
    let history = WebHistory::new(Store::open(&path).unwrap()).unwrap();
    let first = history.sources("one", "turn", "answer").unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].url, "https://one.example/report");
    assert_eq!(first[0].title, "First report");
    assert_eq!(first[0].snippet, "");
    let second = history.sources("two", "turn", "answer").unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].url, "https://two.example/report");
    assert_eq!(second[0].title, "Second report");
    assert_eq!(second[0].snippet, "");
    assert!(
        history
            .sources("unknown", "turn", "answer")
            .unwrap()
            .is_empty()
    );
    assert!(
        history
            .sources("one", "another-turn", "answer")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn deleting_a_conversation_removes_citations_and_sequence_and_refuses_late_writes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("history.sqlite");
    {
        let store = Store::open(&path).unwrap();
        normal(&store, "one");
        normal(&store, "two");
        let history = WebHistory::new(store.clone()).unwrap();
        history
            .record(
                "one",
                "turn",
                "answer",
                vec![source("W1", "https://one.example/report", "First report")],
            )
            .unwrap();
        history.observe_id("one", "W9").unwrap();
        history
            .record(
                "two",
                "turn",
                "answer",
                vec![source("W1", "https://two.example/report", "Second report")],
            )
            .unwrap();
        history.observe_id("two", "W2").unwrap();
        ConversationsRepo::new(&store).remove("one").unwrap();
        history
            .record(
                "one",
                "late-turn",
                "late-answer",
                vec![source("W31", "https://late.example/report", "Late report")],
            )
            .unwrap();
        history.observe_id("one", "W31").unwrap();
    }
    let history = WebHistory::new(Store::open(&path).unwrap()).unwrap();
    assert!(history.sources("one", "turn", "answer").unwrap().is_empty());
    assert!(
        history
            .sources("one", "late-turn", "late-answer")
            .unwrap()
            .is_empty()
    );
    let mut deleted = Vec::new();
    history
        .restore("one", |last, sources| {
            deleted.push((last, sources));
            true
        })
        .unwrap();
    assert_eq!(deleted, vec![(0, vec![])]);
    let mut remaining = Vec::new();
    history
        .restore("two", |last, sources| {
            remaining.push((last, sources));
            true
        })
        .unwrap();
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining[0], (2, vec![]));
    assert_eq!(remaining[1].0, 2);
    assert_eq!(remaining[1].1.len(), 1);
    assert_eq!(remaining[1].1[0].url, "https://two.example/report");
}

#[test]
fn restoration_stops_when_the_consumer_has_no_capacity_and_releases_the_database_lock() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("history.sqlite")).unwrap();
    normal(&store, "one");
    let history = WebHistory::new(store.clone()).unwrap();
    history.observe_id("one", "W3").unwrap();
    history
        .record(
            "one",
            "turn-1",
            "answer-1",
            vec![source("W1", "https://one.example/report", "First report")],
        )
        .unwrap();
    history
        .record(
            "one",
            "turn-2",
            "answer-2",
            vec![source("W2", "https://two.example/report", "Second report")],
        )
        .unwrap();
    history
        .record(
            "one",
            "turn-3",
            "answer-3",
            vec![source("W3", "https://three.example/report", "Third report")],
        )
        .unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let mut calls = Vec::new();
        history
            .restore("one", |last, sources| {
                // A consumer can use the Store without a nested database lock.
                assert!(ConversationsRepo::new(&store).get("one").unwrap().is_some());
                let accepted = sources.is_empty();
                calls.push((last, sources));
                accepted
            })
            .unwrap();
        sender.send(calls).unwrap();
    });
    let calls = receiver
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("restoration must release the Store lock before invoking the consumer");
    worker.join().unwrap();
    assert_eq!(calls.len(), 2, "stop after the first rejected row");
    assert_eq!(calls[0], (3, vec![]));
    assert_eq!(calls[1].0, 3);
    assert_eq!(calls[1].1.len(), 1);
    assert_eq!(calls[1].1[0].source_id, "W1");
    assert_eq!(calls[1].1[0].url, "https://one.example/report");
}
