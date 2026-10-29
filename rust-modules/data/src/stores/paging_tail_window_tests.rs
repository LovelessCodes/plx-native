//! A window at a listing's last page keeps its span. A forward read makes room for its page by
//! shedding half the window, and the page that ended the listing could not use that room, so the
//! shed rows come back: without them the row lost twelve cards on its last page — measured on this
//! household's `home.television.recent` (a 50-row listing, 24-card window), where the ask for the
//! tail at position 49 answered with 13 cards at offset 37 instead of 26 and every card under the
//! focus slid by twelve.

use super::*;

/// A page of `keys` from `start`, one movie per key.
fn page(start: usize, keys: &[usize], total: usize) -> MediaContainer {
    let items: Vec<_> = keys.iter().map(|key| serde_json::json!({
        "ratingKey": key.to_string(), "type": "movie", "title": "Movie", "thumb": "/poster"})).collect();
    serde_json::from_value(serde_json::json!({"offset": start, "totalSize": total, "Metadata": items})).unwrap()
}

fn sid() -> ServerId { ServerId::from_raw(0) }

fn held(keys: std::ops::Range<usize>) -> Vec<Row> {
    keys.map(|i| (i, Arc::new(PmsMovie { rk: i.to_string(), title: "Movie".into(), thumb: "/poster".into(), ..PmsMovie::default() }))).collect()
}

fn keys_of(rows: &[Row]) -> Vec<String> { rows.iter().map(|(_, item)| item.rk.clone()).collect() }

fn listing(total: usize) -> impl FnMut(usize, usize) -> Option<MediaContainer> {
    move |start, size| Some(page(start, &(start..(start + size).min(total)).collect::<Vec<_>>(), total))
}

#[test]
fn a_read_at_a_listings_last_page_keeps_the_span_the_window_had() {
    let total = 50;
    let current = held(25..49);
    let info = PageInfo { offset: 25, end: 49, total, more: true, unstable: false };
    // The row's own next ask: one row is left, and the page brings it with the edge row it overlaps.
    let (rows, info) = fetch_window(sid(), &Ask { start: 49, before: false, hidden: &[] },
        Some((&current, info)), 0, listing(total)).expect("reads");
    assert_eq!(keys_of(&rows), (26..50).map(|i| i.to_string()).collect::<Vec<_>>(),
        "the window keeps its last 24 cards instead of the 13 the tail page re-read");
    assert_eq!((info.offset, info.end, info.more), (26, 50, false));
}

#[test]
fn walking_to_a_listings_end_never_shrinks_the_window() {
    let total = 50;
    let mut rows = held(0..12);
    let mut info = PageInfo { offset: 0, end: 12, total, more: true, unstable: false };
    let mut spans = vec![rows.len()];
    for _ in 0..total {
        if !info.more { break; }
        let start = info.end.max(info.offset + rows.len());
        let (next, next_info) = fetch_window(sid(), &Ask { start, before: false, hidden: &[] },
            Some((&rows, info)), 0, listing(total)).expect("reads");
        rows = next;
        info = next_info;
        spans.push(rows.len());
    }
    assert_eq!(keys_of(&rows).last().unwrap(), "49", "the walk reaches the listing's last row");
    assert!(!info.more);
    assert!(spans.iter().all(|span| *span <= 24));
    assert!(spans.iter().skip_while(|span| **span < 24).all(|span| *span == 24),
        "the window grew to 24 and never gave cards back: {spans:?}");
}
