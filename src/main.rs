#![allow(unused)]
fn ratelimiter_with_interval(interval_sec: u64) -> governor::DefaultDirectRateLimiter {
    governor::RateLimiter::direct(
        governor::Quota::with_period(
            std::time::Duration::from_secs(interval_sec)).expect("failed to setup ratelimiter")
    )
}

async fn clean_the_windows(_task_id: usize, _robot_name: &str) -> String {
    // Simulated execution time (0.3 seconds)
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    String::from("Squeeesh")
}

async fn water_the_plants(_task_id: usize, _robot_name: &str) -> String {
    // Simulated execution time (0.7 seconds)
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    String::from("Blub")
}

async fn feed_the_cat(_task_id: usize, _robot_name: &str) -> String {
    // Simulated execution time (0.5 seconds)
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    String::from("Meow")
}

async fn execute_task(task_name: &str, task_id: usize, robot_name: &str) -> String {
    match task_name {
        "clean_the_windows" => clean_the_windows(task_id, robot_name).await,
        "water_the_plants" => water_the_plants(task_id, robot_name).await,
        "feed_the_cat" => feed_the_cat(task_id, robot_name).await,
        _ => panic!("invalid task_name")
    }
}

// there are two ways to solve this problem
// one is more idiomatic to rust, but not as deterministic and thus the ordering of task will not be optimal
// the second will be more deterministic and will try to optimize the ordering of tasks to avoid
// idle time due to waiting for ratelimiters
//
// it is possible to compute the IDEAL ordering of tasks, but it's a NP hard problem you'd
// typically never implement a solution for in the real world, especially whena a non ideal but
// optimized implementation could get pretty close

// idiomatic use automatic scheduling, ie, i don't manually manage the ordering of tasks
mod idiomatic {
    use std::collections::HashMap;
    use tokio::sync::mpsc::unbounded_channel;

    pub async fn solve(tasks: Vec<(usize, &str, &str)>) {
        let mut robots_senders = HashMap::new();
        let mut handles = Vec::new();

        let task_config = std::sync::Arc::new(HashMap::from([
                ("clean_the_windows", super::ratelimiter_with_interval(5)),
                ("water_the_plants", super::ratelimiter_with_interval(3)),
                ("feed_the_cat", super::ratelimiter_with_interval(2)),
        ]));

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(3)); // concurency of 3

        for robot_name in ["Dave", "Cris", "Andi", "Nick", "Phil", "Maxi"] { // prepare execution context
            let (tx, mut rx) = unbounded_channel::<(usize, String)>();
            robots_senders.insert(robot_name, tx);

            let sem = semaphore.clone();
            let task_config = task_config.clone();
            let handle = tokio::task::spawn(async move {
                while let Some((task_id, task)) = rx.recv().await {
                    let Some(rt) = task_config.get(task.as_str()) else {
                        println!("invalid task name : {task}");
                        continue;
                    };
                    println!("{robot_name} waiting for {task} with id {task_id}");
                    rt.until_ready().await; // waiting on the ratelimiter
                    let _permit = sem.acquire().await; // to limit concurency accross robots
                    println!("{robot_name} started {task} with id {task_id}");
                    super::execute_task(&task, task_id, robot_name).await;
                    println!("{robot_name} finished {task} with id {task_id}")
                }
                println!("robot : {robot_name} finished working")
            });
            handles.push(handle);
        }

        // dispatch the tasks to the robots

        for (id, robot, task_name) in tasks {
            let robot_handle = robots_senders.get(robot).expect(&format!("unknown robot {robot}"));
            robot_handle.send((id, task_name.into())).expect("failed to send task");
        }

        drop(robots_senders); // droping the senders so the handles can end, this could be optional if we
                              // wanted to add more tasks as we are going
        futures::future::try_join_all(handles).await;
        println!("all tasks have been done")
    }
}


// TODO manual scheduling, ordering of tasks is manual and optimized to minimize waiting time due to
// ratelimits, i can write it if you ask for it
mod optimized {
}

#[tokio::main]
async fn main() {
    let tasks = vec![
        (1, "Dave", "clean_the_windows"),
        (2, "Dave", "water_the_plants"),
        (3, "Dave", "clean_the_windows"),
        (4, "Dave", "feed_the_cat"),
        (5, "Dave", "clean_the_windows"),
        (6, "Cris", "water_the_plants"),
        (7, "Cris", "clean_the_windows"),
        (8, "Cris", "clean_the_windows"),
        (9, "Cris", "feed_the_cat"),
        (10, "Cris", "water_the_plants"),
        (11, "Andi", "clean_the_windows"),
        (12, "Andi", "water_the_plants"),
        (13, "Andi", "clean_the_windows"),
        (14, "Andi", "feed_the_cat"),
        (15, "Andi", "clean_the_windows"),
        (16, "Nick", "water_the_plants"),
        (17, "Nick", "clean_the_windows"),
        (18, "Nick", "clean_the_windows"),
        (19, "Nick", "feed_the_cat"),
        (20, "Nick", "water_the_plants"),
        (21, "Phil", "clean_the_windows"),
        (22, "Phil", "water_the_plants"),
        (23, "Phil", "clean_the_windows"),
        (24, "Phil", "feed_the_cat"),
        (25, "Phil", "clean_the_windows"),
        (26, "Maxi", "water_the_plants"),
        (27, "Maxi", "clean_the_windows"),
        (28, "Maxi", "clean_the_windows"),
        (29, "Maxi", "feed_the_cat"),
        (30, "Maxi", "water_the_plants")
    ];
    idiomatic::solve(tasks).await;
}
