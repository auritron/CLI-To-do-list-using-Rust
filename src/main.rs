use std::io::stdin;
use std::fs;
use std::fmt;
use std::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct ListItem {
    id: u32,
    title:String,
    status:Status,
}

#[derive(Debug, Serialize, Deserialize)]
enum Status {
    Pending,
    CheckAgain,
    Completed,
}

#[derive(Debug)]
enum TaskError {
    IDNotFound,
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::IDNotFound => write!(f, "Task with the given ID was not found"),
        }
    }
}

enum Command {
    ShowTasks,
    SaveTasks,
    AddTask,
    RenameTask,
    UpdateTask,
    DeleteTask,
    CloseProgram,
    Null,
}

fn save_tasks(tasks: &[ListItem], filepath: &str) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(tasks)?;
    fs::write(filepath, json)?;
    Ok(())
}

fn load_tasks(filepath: &str) -> Result<Vec<ListItem>, Box<dyn Error>> {
    if !std::path::Path::new(filepath).exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(filepath)?;
    let tasks: Vec<ListItem> = serde_json::from_str(&contents).unwrap();
    Ok(tasks)
}

fn add_task(tasks: &mut Vec<ListItem>, title: &str) -> () {
    let next_id = tasks
                    .iter()
                    .map(|t| t.id)
                    .max()
                    .unwrap_or(0) + 1;
    tasks.push(ListItem {
        id: next_id,
        title: title.to_string(),
        status: Status::Pending,
    });
}

fn get_task<'a>(tasks: &'a mut [ListItem], id: u32) -> Result<&'a mut ListItem, TaskError> {
    tasks
                    .iter_mut()
                    .find(|t| t.id == id)
                    .ok_or(TaskError::IDNotFound)
}

fn edit_task_title(tasks: &mut [ListItem], id: u32, new_title: &str) -> Result<(), TaskError> {
    let target = get_task(tasks, id)?;
    target.title  = new_title.to_string();
    Ok(())
}

fn remove_task(tasks: &mut Vec<ListItem>, id: u32) -> Result<(), TaskError> { 
    let target_index = tasks 
                                .iter() 
                                .position(|t| t.id == id) 
                                .ok_or(TaskError::IDNotFound)?; 
    tasks.swap_remove(target_index); 
    Ok(()) 
}

fn update_status(target: &mut ListItem, new_status: Status) -> () {
    target.status = new_status;
}

fn display_tasks(tasks: &[ListItem]) {
    println!("--------------------------------------------------");
    println!("S.No. | ID | Task | Status");
    for (i, t) in tasks
                                    .iter()
                                    .enumerate()
                                    .map(|(i, t)| (i as u32 + 1, t)) 
    {
        let status = match t.status {
            Status::Pending => "Pending",
            Status::CheckAgain => "Check Again",
            Status::Completed => "Completed"
        };
        println!("{} | {} | {} | {}", i, t.id, t.title, status);
    }
    println!("--------------------------------------------------");
}

fn main() {
    let mut utrim_command = String::from("");
    let mut target_id = String::from("");
    let mut new_title = String::from("");
    let mut new_status = String::from("");

    println!("Welcome to the CLI Task Viewer!");
    println!("Loading Tasks...");
    let mut tasklist = match load_tasks("tasks.json") {
        Ok(tasks) => tasks,
        Err(e) => { 
            eprintln!("Error in reading file: {}", e);
            std::process::exit(1);
        }
    };
    println!("Tasks loaded!");
    println!("--------------------------------------------------");
    println!("List of tasks for the day:");
    display_tasks(&tasklist);
    loop {
        utrim_command.clear();
        println!("Enter a command to manage your todo list: (show/add/remove/update/rename/close)");
        stdin()
            .read_line(&mut utrim_command)
            .expect("Failed to read command.");
        let command_str = utrim_command
                                    .trim()
                                    .to_lowercase();
        let command = match command_str.as_str() {
            "show" => Command::ShowTasks,
            "add" => Command::AddTask,
            "remove" | "delete" => Command::DeleteTask,
            "update" => Command::UpdateTask,
            "rename" => Command::RenameTask,
            "close" => Command::CloseProgram,
            _ => {
                println!("Unrecognized command! Try again.");
                Command::Null
            }
        };

        match command {
            Command::ShowTasks => {
                display_tasks(&tasklist);
            },
            Command::AddTask => {
                new_title.clear();
                println!("Name the task to be added.");
                stdin()
                    .read_line(&mut new_title)
                    .expect("Failed to read text.");
                add_task(&mut tasklist, new_title.trim());
                println!("New task added successfully!");
            },
            Command::DeleteTask => {
                target_id.clear();
                println!("Enter the target ID of the task you want to delete.");
                stdin()
                    .read_line(&mut target_id)
                    .expect("Failed to read text.");
                match remove_task(&mut tasklist, match target_id
                    .trim()
                    .parse() {
                        Ok(id) => id,
                        Err(e) => {
                            println!("Failed to read ID correctly: {}", e);
                            continue;
                        }
                }) {
                    Ok(()) => println!("Successfully removed task!"),
                    Err(e) => eprintln!("Error, failed to remove task: {}", e)
                };
            }
            Command::UpdateTask => {
                target_id.clear();
                new_status.clear();
                println!("Enter the target ID of the task you want to update.");
                stdin()
                    .read_line(&mut target_id)
                    .expect("Failed to read text.");
                let target_id_num: u32 = match target_id
                    .trim()
                    .parse() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("Failed to read ID correctly: {}", e);
                            continue;
                        }
                    };
                let target_task = match get_task(&mut tasklist, target_id_num) {
                    Ok(task) => {
                        println!(
                            "Now, enter the new status of task '{}'. (complete/pending/check again)",
                            task.title
                        );
                        task
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                };
                stdin()
                    .read_line(&mut new_status)
                    .expect("Failed to read text.");
                let target_status = new_status
                    .trim()
                    .to_lowercase();
                match target_status.as_str() {
                    "complete" => {
                        update_status(target_task, Status::Completed);
                        println!("The task \'{}\' was successfully completed.", target_task.title);
                    },
                    "pending" => {
                        update_status(target_task,  Status::Pending);
                        println!("The task \'{}\' is now pending.", target_task.title);
                    },
                    "check again" | "checkagain" => {
                        update_status(target_task, Status::CheckAgain);
                        println!("The task \'{}\' is now set for rechecking.", target_task.title);
                    },
                    _ => {
                        println!("Unknown status. Please try again.");
                        continue;
                    }
                }
            },
            Command::RenameTask => {
                target_id.clear();
                new_title.clear();
                println!("Enter the target ID of the task you want to rename.");
                stdin()
                    .read_line(&mut target_id)
                    .expect("Failed to read text.");
                let target_id_num: u32 = match target_id
                    .trim()
                    .parse() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("Failed to read ID correctly: {}", e);
                            continue;
                        }
                    };
                let mut target_task = get_task(&mut tasklist, target_id_num);
                match target_task {
                    Ok(task) => {
                        println!("Now, enter the name you want to change \'{}\' to.", task.title);
                    },
                    Err(e) => {
                        println!("Oops! Something went wrong: {}", e);
                        continue;
                    }
                };
                stdin()
                    .read_line(&mut new_title)
                    .expect("Failed to read text.");
                edit_task_title(&mut tasklist, target_id_num, new_title.trim());
                println!("Task named changed successfully!")
            },
            Command::SaveTasks => {
                println!("Saving tasks...");
                match save_tasks(&tasklist, "tasks.json") {
                    Ok(()) => println!("Successfully saved tasks!"),
                    Err(_e) => eprintln!("Failed to save tasks correctly. Please try again.")
                }
            }
            Command::CloseProgram => {
                println!("Saving tasks before closing...");
                match save_tasks(&tasklist, "tasks.json") {
                    Ok(()) => {
                        println!("Successfully saved tasks! Closing program...");
                        std::process::exit(0);
                    },
                    Err(_e) => {
                        eprintln!("Failed to save tasks correctly. Process will not exit if tasks are not saved.")
                    }
                }
            }
            Command::Null => {
                println!("Oops, that's not a recognized command. Try again.")
            }
        }
    }
}