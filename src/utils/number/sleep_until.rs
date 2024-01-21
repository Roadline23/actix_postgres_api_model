use std::time::Duration;
use chrono::Utc;
use tokio::time::sleep;

pub async fn sleep_until(scheduled_time: i64) {
    let now = Utc::now().timestamp();
    let exp_in = scheduled_time - now;

    if exp_in > 0 {
        while Utc::now().timestamp() < scheduled_time {
            let sleep_duration = Duration::from_secs(exp_in as u64);
            sleep(sleep_duration).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_sleeps_until() {
        let now = Utc::now().timestamp();
        let scheduled_time = now + 5;
        let exp_in = scheduled_time - now;

        assert_eq!(exp_in, 5);

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            sleep_until(scheduled_time).await;
        });
    }

    #[test]
    fn it_does_not_sleep_until() {
        let now = Utc::now().timestamp();
        let scheduled_time = now - 5;
        let exp_in = scheduled_time - now;

        assert_eq!(exp_in, -5);

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            sleep_until(scheduled_time).await;
        });
    }
}
