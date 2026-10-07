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
pub fn missed(items: &[Item], now: DateTime<Utc>, rules: &Rules) -> Vec<MissedItem> {
    let mut result: Vec<_> = items
        .iter()
        .filter_map(|i| unanswered(i, now, rules))
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
}
