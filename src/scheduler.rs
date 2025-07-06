use std::{collections::HashMap, sync::mpsc::Sender, thread, time::Duration};
use crate::world_command::WorldCommand;

pub fn start_scheduler(tx: &Sender<WorldCommand>) {
  let scheduler_sender = tx.clone();

    thread::spawn(move || {
        let mut tasks: HashMap<u32, Box<dyn Fn()>> = HashMap::new();
    
        tasks.insert(30, Box::new(||{
            let commands = scheduler_sender.clone();
            let _ = commands.send(WorldCommand::SystemMessage { message: String::from("World Saved.") });
            let _ = commands.send(WorldCommand::Save);
        }));

        let mut time: u32 = 0;
        loop {
            for (scheduled_time, task) in &tasks {
                if time % scheduled_time == 0 {
                    task();
                }
            }

            thread::sleep(Duration::from_secs(1));
            time += 1;
        }        
    });
}