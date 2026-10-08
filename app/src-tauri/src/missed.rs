use crate::model::{Item, ItemKind};
use chrono::{DateTime, Duration, Local, Utc};
use serde::Serialize;
pub struct Rules {
    pub mail_unanswered_days: u32,
    pub github_review_days: u32,
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
fn overdue_task(item: &Item, now: DateTime<Utc>) -> Option<MissedItem> {
    let due = item.due?;
    if item.kind != ItemKind::Task || item.done || due >= now {
        return None;
    }
    Some(MissedItem {
        item: item.clone(),
        reason: format!(
            "마감 {}일 지남",
            (now.with_timezone(&Local).date_naive() - due.with_timezone(&Local).date_naive())
                .num_days()
                .max(1)
        ),
        since: due,
    })
}
fn review_requested(item: &Item, now: DateTime<Utc>, rules: &Rules) -> Option<MissedItem> {
    if item.kind != ItemKind::PullRequest || item.meta.get("role")?.as_str()? != "reviewer" {
        return None;
    }
    let since = DateTime::parse_from_rfc3339(item.meta.get("requested_at")?.as_str()?)
        .ok()?
        .with_timezone(&Utc);
    let age = now - since;
    (age >= Duration::days(rules.github_review_days.into())).then(|| MissedItem {
        item: item.clone(),
        reason: format!("리뷰 요청 {}일째", age.num_days()),
        since,
    })
}
fn overdue_issue(item: &Item, now: DateTime<Utc>) -> Option<MissedItem> {
    let due = item.due?;
    if item.kind != ItemKind::Issue || due >= now {
        return None;
    }
    Some(MissedItem {
        item: item.clone(),
        reason: format!(
            "마일스톤 기한 {}일 지남",
            (now.with_timezone(&Local).date_naive() - due.with_timezone(&Local).date_naive())
                .num_days()
                .max(1)
        ),
        since: due,
    })
}
pub fn missed(items: &[Item], now: DateTime<Utc>, rules: &Rules) -> Vec<MissedItem> {
    let mut result: Vec<_> = items
        .iter()
        .filter_map(|i| {
            unanswered(i, now, rules)
                .or_else(|| unanswered_invitation(i, now))
                .or_else(|| overdue_task(i, now))
                .or_else(|| review_requested(i, now, rules))
                .or_else(|| overdue_issue(i, now))
        })
        .collect();
    result.sort_by_key(|i| i.since);
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::fixture;
    use chrono::TimeZone;
    #[test]
    fn boundary_sender_and_order() {
        let now = Utc::now();
        let rules = Rules {
            mail_unanswered_days: 2,
            github_review_days: 1,
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
                github_review_days: 1,
            },
        );
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].item.id, "future");
        assert_eq!(got[0].since, created);
        assert_eq!(got[0].reason, "초대에 응답 안 함");
    }
    #[test]
    fn overdue_incomplete_tasks_are_missed() {
        let now = Utc::now();
        let mut overdue = fixture("overdue", ItemKind::Task, now);
        overdue.due = Some(now - Duration::days(2));
        let mut done = overdue.clone();
        done.done = true;
        let mut no_due = overdue.clone();
        no_due.due = None;
        let mut upcoming = overdue.clone();
        upcoming.due = Some(now + Duration::seconds(1));
        let got = missed(
            &[done, no_due, upcoming, overdue],
            now,
            &Rules {
                mail_unanswered_days: 2,
                github_review_days: 1,
            },
        );
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].reason, "마감 2일 지남");
        assert_eq!(got[0].since, now - Duration::days(2));
    }
    #[test]
    fn task_due_yesterday_counts_as_one_day() {
        let now = Local.with_ymd_and_hms(2026, 10, 8, 9, 0, 0).unwrap();
        let due = Local.with_ymd_and_hms(2026, 10, 7, 23, 59, 59).unwrap();
        let mut task = fixture("yesterday", ItemKind::Task, now.with_timezone(&Utc));
        task.due = Some(due.with_timezone(&Utc));
        let got = missed(
            &[task],
            now.with_timezone(&Utc),
            &Rules {
                mail_unanswered_days: 2,
                github_review_days: 1,
            },
        );
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].reason, "마감 1일 지남");
    }
    #[test]
    fn review_request_boundary_and_role() {
        let now = Utc::now();
        let rules = Rules {
            mail_unanswered_days: 2,
            github_review_days: 1,
        };
        let mut edge = fixture("edge", ItemKind::PullRequest, now);
        edge.meta = serde_json::json!({"role":"reviewer", "requested_at":(now - Duration::days(1)).to_rfc3339()});
        let mut young = edge.clone();
        young.id = "young".into();
        young.meta["requested_at"] = (now - Duration::days(1) + Duration::seconds(1))
            .to_rfc3339()
            .into();
        let mut author = edge.clone();
        author.id = "author".into();
        author.meta["role"] = "author".into();
        let got = missed(&[young, author, edge], now, &rules);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].item.id, "edge");
        assert_eq!(got[0].reason, "리뷰 요청 1일째");
        assert_eq!(got[0].since, now - Duration::days(1));
    }
    #[test]
    fn issue_due_boundary() {
        let now = Utc::now();
        let rules = Rules {
            mail_unanswered_days: 2,
            github_review_days: 1,
        };
        let mut overdue = fixture("overdue", ItemKind::Issue, now);
        overdue.due = Some(now - Duration::seconds(1));
        let mut at_due = overdue.clone();
        at_due.id = "at_due".into();
        at_due.due = Some(now);
        let got = missed(&[at_due, overdue], now, &rules);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].reason, "마일스톤 기한 1일 지남");
        assert_eq!(got[0].since, now - Duration::seconds(1));
    }
}
