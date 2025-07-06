use std::{collections::HashMap, sync::mpsc::Sender, thread, time::Duration};
use crate::world_command::WorldCommand;

type Task = Box<dyn Fn() + Send + 'static>;

pub struct Scheduler {
  pub tasks: HashMap<u32, Task>
}

impl Scheduler {
  pub fn schedule_all_default(&mut self, tx: Sender<WorldCommand>) {
    let scheduler_sender: Sender<WorldCommand> = tx.clone();

    //World saving task (Every 30 seconds.)
    self.schedule(30, Box::new(move||{
      let commands = scheduler_sender.clone();
      let _ = commands.send(WorldCommand::SystemMessage { message: String::from("World Saved.") });
      let _ = commands.send(WorldCommand::Save);
    }));
  }
  
  pub fn schedule(&mut self, time: u32, task: Task) {
    let _ = &self.tasks.insert(time, task);
  }

  pub fn start_scheduler(self) {
    let tasks = self.tasks;
    thread::spawn(move||{
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
}
