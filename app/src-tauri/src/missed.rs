use crate::model::{Item, ItemKind};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
pub struct Rules {
    pub mail_unanswered_days: u32,
}
#[derive(Clone, Debug, Serialize)]
pub struct MissedItem {
    pub item: Item,
    pub reason: String,
    pub since: DateTime<Utc>,
}
fn unanswered(item: &Item, now: DateTime<Utc>, rules: &Rules) -> Option<MissedItem> {
    let age = now - item.timestamp;
    (item.kind == ItemKind::Email
        && item.meta.get("last_from_me").and_then(|v| v.as_bool()) == Some(false)
        && age >= Duration::days(rules.mail_unanswered_days.into()))
    .then(|| MissedItem {
        item: item.clone(),
        reason: format!("{}일째 답장 안 함", age.num_days()),
        since: item.timestamp,
    })
}
fn unanswered_invitation(item: &Item, now: DateTime<Utc>) -> Option<MissedItem> {
    if item.kind != ItemKind::Event
        || item.timestamp <= now
        || item.meta.get("my_response").and_then(|v| v.as_str()) != Some("needsAction")
    {
        return None;
    }
    let since = DateTime::parse_from_rfc3339(item.meta.get("created")?.as_str()?)
        .ok()?
        .with_timezone(&Utc);
    Some(MissedItem {
        item: item.clone(),
        reason: "초대에 응답 안 함".into(),
        since,
    })
}
pub fn missed(items: &[Item], now: DateTime<Utc>, rules: &Rules) -> Vec<MissedItem> {
    let mut result: Vec<_> = items
        .iter()
        .filter_map(|i| unanswered(i, now, rules).or_else(|| unanswered_invitation(i, now)))
        .collect();
    result.sort_by_key(|i| i.since);
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::fixture;
    #[test]
    fn boundary_sender_and_order() {
        let now = Utc::now();
        let rules = Rules {
            mail_unanswered_days: 2,
        };
        let young = fixture("young", ItemKind::Email, now - Duration::hours(47));
        let edge = fixture("edge", ItemKind::Email, now - Duration::days(2));
        let old = fixture("old", ItemKind::Email, now - Duration::days(4));
        let mut mine = old.clone();
        mine.meta["last_from_me"] = true.into();
        let got = missed(&[young, edge, old, mine], now, &rules);
        assert_eq!(
            got.iter().map(|m| m.item.id.as_str()).collect::<Vec<_>>(),
            vec!["old", "edge"]
        );
        assert_eq!(got[1].reason, "2일째 답장 안 함");
    }
    #[test]
    fn only_future_unanswered_invitations_are_missed() {
        let now = Utc::now();
        let created = now - Duration::days(3);
        let mut future = fixture("future", ItemKind::Event, now + Duration::hours(1));
        future.meta =
            serde_json::json!({"my_response":"needsAction", "created":created.to_rfc3339()});
        let mut past = future.clone();
        past.timestamp = now - Duration::seconds(1);
        let mut starting = future.clone();
        starting.timestamp = now;
        let mut accepted = future.clone();
        accepted.meta["my_response"] = "accepted".into();
        let got = missed(
            &[past, starting, accepted, future],
            now,
            &Rules {
                mail_unanswered_days: 2,
            },
        );
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].item.id, "future");
        assert_eq!(got[0].since, created);
        assert_eq!(got[0].reason, "초대에 응답 안 함");
    }
}
