extern crate chrono;

use std::fmt;
use chrono::{Duration, NaiveDateTime};

use crate::store::{Entry};

/**
 * Activity: Duration of all Entry's with the same task
 */
pub struct Activity {
    name: String,
    duration: Duration,
}

impl fmt::Display for Activity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:>2} h {:>2} min: {}", self.duration.num_hours(), self.duration.num_minutes() % 60, self.name)
    }
}

/**
 * Activities: Collection of Activity with total durations
 */
pub struct Activities {
    activities: Vec<Activity>,
    total_work: Duration,
    total_slack: Duration,
}

impl Activities {
    pub fn new_from_entries<'a>(entries: impl Iterator<Item = &'a Entry>) -> Activities {
        // don't use a hashmap here, we do want to keep this sorted by "first occurrence of task"
        let mut activities = Vec::new();
        let mut total_work = Duration::minutes(0);
        let mut total_slack = Duration::minutes(0);
        let mut prev_stop: Option<NaiveDateTime> = None;

        for entry in entries {
            // first entry's task is ignored, it just provides the start time
            if prev_stop.is_none() {
                prev_stop = Some(entry.stop);
                continue;
            }
            let duration = entry.stop.signed_duration_since(prev_stop.unwrap());
            if entry.task.starts_with("**") {
                total_slack = total_slack + duration;
            } else {
                total_work = total_work + duration;
            }

            // meh quadratic loop, but not important
            match activities.iter_mut().find(|a: &&mut Activity| a.name == entry.task) {
                Some(a) => { a.duration = a.duration + duration },
                None => activities.push(Activity { name: entry.task.to_string(), duration }),
            }

            prev_stop = Some(entry.stop);
        }

        Activities { activities, total_work, total_slack }
    }
}

impl fmt::Display for Activities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for a in &self.activities {
            writeln!(f, "{}", a)?;
        }
        writeln!(f, "-------")?;
        writeln!(f, "Total work done: {} h {} min", self.total_work.num_hours(), self.total_work.num_minutes() % 60)?;
        writeln!(f, "Total slacking: {} h {} min", self.total_slack.num_hours(), self.total_slack.num_minutes() % 60)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate};
    use crate::store::Timelog;

    const DAY_LOG: &'static str = "
2022-06-10 07:00: arrived
2022-06-10 08:45: gtimelog: code
2022-06-10 09:00: ** tea
2022-06-10 12:05: gtimelog: code
2022-06-10 12:35: customer joe: inquiry
2022-06-10 13:15: ** lunch
2022-06-10 14:00: code
2022-06-10 15:00: bug triage
2022-06-10 15:10: ** tea
2022-06-10 16:00: customer joe: support
";


    #[test]
    fn test_activity_display() {
        assert_eq!(&format!("{}", Activity { name: "code this".to_string(), duration: Duration::minutes(3) }),
                   " 0 h  3 min: code this");
        assert_eq!(&format!("{}", Activity { name: "code this".to_string(), duration: Duration::minutes(59) }),
                   " 0 h 59 min: code this");
        assert_eq!(&format!("{}", Activity { name: "code this".to_string(), duration: Duration::minutes(60) }),
                   " 1 h  0 min: code this");
        assert_eq!(&format!("{}", Activity { name: "code this".to_string(), duration: Duration::minutes(23 * 60 + 1) }),
                   "23 h  1 min: code this");
    }

    #[test]
    fn test_activities_construct() {
        let a = Activities::new_from_entries(vec![].iter());
        assert_eq!(a.activities.len(), 0);
        assert_eq!(a.total_work, Duration::minutes(0));
        assert_eq!(a.total_slack, Duration::minutes(0));

        let tl = Timelog::new_from_string(DAY_LOG);
        let a = Activities::new_from_entries(tl.get_day(&NaiveDate::from_ymd(2022, 6, 10)));
        assert_eq!(a.total_work, Duration::minutes(475));
        assert_eq!(a.total_slack, Duration::minutes(65));
        assert_eq!(a.activities.len(), 7);
        assert_eq!(a.activities[0].name, "gtimelog: code");
        // first block 1:45, second block 3:05
        assert_eq!(a.activities[0].duration, Duration::hours(4) + Duration::minutes(50));

        assert_eq!(format!("{}", a),
" 4 h 50 min: gtimelog: code
 0 h 25 min: ** tea
 0 h 30 min: customer joe: inquiry
 0 h 40 min: ** lunch
 0 h 45 min: code
 1 h  0 min: bug triage
 0 h 50 min: customer joe: support
-------
Total work done: 7 h 55 min
Total slacking: 1 h 5 min\n")
    }
}