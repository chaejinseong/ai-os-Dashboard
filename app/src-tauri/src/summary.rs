use crate::{
    missed::MissedItem,
    model::{Item, ItemKind, Stage},
};
use chrono::{DateTime, Datelike, Duration, Local};
use serde::Serialize;
#[derive(Default, Serialize)]
pub struct Completion {
    pub done: usize,
    pub left: usize,
}
#[derive(Default, Serialize)]
pub struct Board {
    pub today: usize,
    pub this_week: usize,
    pub later: usize,
}
#[derive(Default, Serialize)]
pub struct Pipeline {
    pub requested: usize,
    pub in_progress: usize,
    pub review: usize,
    pub done: usize,
    pub missed_by_stage: [usize; 4],
}
#[derive(Default, Serialize)]
pub struct Summary {
    pub completion: Completion,
    pub board: Board,
    pub pipeline: Pipeline,
    pub schedule: Vec<Item>,
    pub missed_count: usize,
    pub open_work_count: usize,
}
pub fn summarize(
    items: &[Item],
    missed: &[MissedItem],
    now: DateTime<Local>,
    period_days: u32,
) -> Summary {
    let today = now.date_naive();
    let start = today - Duration::days(i64::from(period_days.clamp(1, 30)) - 1);
    let sunday = today + Duration::days(i64::from(6 - today.weekday().num_days_from_monday()));
    let in_period = |item: &Item| {
        item.due.is_some_and(|d| {
            let day = d.with_timezone(&Local).date_naive();
            day >= start && day <= today
        })
    };
    let mut s = Summary {
        missed_count: missed.len(),
        ..Default::default()
    };
    for i in items {
        if i.kind == ItemKind::Task && in_period(i) {
            if i.done {
                s.completion.done += 1
            } else {
                s.completion.left += 1
            }
        }
        if matches!(i.kind, ItemKind::Task | ItemKind::Issue) && !i.done {
            match i.due.map(|d| d.with_timezone(&Local).date_naive()) {
                Some(d) if d <= today => s.board.today += 1,
                Some(d) if d <= sunday => s.board.this_week += 1,
                _ => s.board.later += 1,
            }
        }
        match i.stage {
            Some(Stage::Requested) => s.pipeline.requested += 1,
            Some(Stage::InProgress) => s.pipeline.in_progress += 1,
            Some(Stage::Review) => s.pipeline.review += 1,
            Some(Stage::Done) if in_period(i) => s.pipeline.done += 1,
            _ => {}
        }
        if i.kind == ItemKind::Event {
            let day = i.timestamp.with_timezone(&Local).date_naive();
            if day == today || day == today + Duration::days(1) {
                s.schedule.push(i.clone());
            }
        }
        if matches!(
            i.kind,
            ItemKind::Task | ItemKind::Issue | ItemKind::PullRequest
        ) && !i.done
        {
            s.open_work_count += 1
        }
    }
    for m in missed {
        let stage = match m.item.stage {
            Some(Stage::Requested) => 0,
            Some(Stage::InProgress) => 1,
            Some(Stage::Review) => 2,
            Some(Stage::Done) if in_period(&m.item) => 3,
            _ => continue,
        };
        s.pipeline.missed_by_stage[stage] += 1;
    }
    s.schedule.sort_by_key(|i| i.timestamp);
    s
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::fixture;
    use chrono::{TimeZone, Utc};
    fn local(day: u32, hour: u32, min: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 10, day, hour, min, 0).unwrap()
    }
    #[test]
    fn boundaries_board_pipeline_and_empty() {
        let now = local(7, 12, 0);
        let mut items = Vec::new();
        for (id, date, stage, done) in [
            ("included", Some(local(1, 0, 0)), Stage::Done, true),
            (
                "excluded",
                Some(Local.with_ymd_and_hms(2026, 9, 30, 23, 59, 0).unwrap()),
                Stage::Done,
                true,
            ),
            ("today", Some(local(7, 23, 59)), Stage::Requested, false),
            ("sunday", Some(local(11, 23, 59)), Stage::InProgress, false),
            ("later", Some(local(12, 0, 0)), Stage::Review, false),
            ("none", None, Stage::Requested, false),
        ] {
            let mut i = fixture(id, ItemKind::Task, now.with_timezone(&Utc));
            i.due = date.map(|d| d.with_timezone(&Utc));
            i.stage = Some(stage);
            i.done = done;
            items.push(i);
        }
        let missed = vec![MissedItem {
            item: items[2].clone(),
            reason: "test".into(),
            since: now.with_timezone(&Utc),
        }];
        let s = summarize(&items, &missed, now, 7);
        assert_eq!((s.completion.done, s.completion.left), (1, 1));
        assert_eq!((s.board.today, s.board.this_week, s.board.later), (1, 1, 2));
        assert_eq!(
            (
                s.pipeline.requested,
                s.pipeline.in_progress,
                s.pipeline.review,
                s.pipeline.done
            ),
            (2, 1, 1, 1)
        );
        assert_eq!(s.pipeline.missed_by_stage, [1, 0, 0, 0]);
        assert_eq!(s.open_work_count, 4);
        let empty = summarize(&[], &[], now, 7);
        assert_eq!(
            empty.completion.done + empty.completion.left + empty.missed_count,
            0
        );
        assert!(empty.schedule.is_empty());
        items[2].due = Some(local(6, 0, 0).with_timezone(&Utc));
        assert_eq!(summarize(&items, &[], now, 1).board.today, 1);
    }
    #[test]
    fn schedule_local_dates_sorted() {
        let now = local(7, 12, 0);
        let items = [
            fixture(
                "tomorrow",
                ItemKind::Event,
                local(8, 23, 59).with_timezone(&Utc),
            ),
            fixture("today", ItemKind::Event, local(7, 0, 0).with_timezone(&Utc)),
            fixture("later", ItemKind::Event, local(9, 0, 0).with_timezone(&Utc)),
        ];
        assert_eq!(
            summarize(&items, &[], now, 7)
                .schedule
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            vec!["today", "tomorrow"]
        );
    }
}
