use chrono::{Local, FixedOffset, TimeZone, Duration, Timelike};

fn now() -> String {
  Local::now().format("%FT%T.%3f").to_string()
}

#[tokio::main]
async fn main() {
  let dt = FixedOffset::east(8 * 3600).ymd(2021, 12, 20).and_hms(9, 32, 0);
  println!("{:?}", dt.format("%FT%T%z").to_string());
  println!("{:?}", dt.timestamp());
  let x = Local::now() + Duration::seconds(1);
  println!("{:?}", x);
  println!("{}", chrono::Utc::now().second());
  // let x = dt.signed_duration_since(Local::now());
  // let x = Local::now().signed_duration_since(dt);
  // println!("{:?}", x.num_milliseconds());
  tick().await;
}

async fn tick() {
  let mut intv = tokio::time::interval(std::time::Duration::from_secs(5));
  loop {
    intv.tick().await;
    println!("{}", now());
  }
}



