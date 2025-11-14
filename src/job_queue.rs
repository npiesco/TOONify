use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub operation: String,
    pub data: String,
    pub status: JobStatus,
    pub result: Option<String>,
    pub error: Option<String>,
}

pub type JobStore = Arc<Mutex<HashMap<String, Job>>>;

pub fn create_job_store() -> JobStore {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn submit_job(store: JobStore, operation: String, data: String) -> String {
    let job_id = Uuid::new_v4().to_string();
    
    let job = Job {
        id: job_id.clone(),
        operation,
        data,
        status: JobStatus::Pending,
        result: None,
        error: None,
    };
    
    eprintln!("[JOB QUEUE] Submitted job: {}", job_id);
    
    let mut jobs = store.lock().unwrap();
    jobs.insert(job_id.clone(), job);
    
    job_id
}

pub fn get_job_status(store: JobStore, job_id: &str) -> Option<(JobStatus, Option<String>)> {
    let jobs = store.lock().unwrap();
    jobs.get(job_id).map(|job| (job.status.clone(), job.error.clone()))
}

pub fn get_job_result(store: JobStore, job_id: &str) -> Option<String> {
    let jobs = store.lock().unwrap();
    jobs.get(job_id).and_then(|job| job.result.clone())
}

pub fn list_jobs(store: JobStore) -> Vec<Job> {
    let jobs = store.lock().unwrap();
    jobs.values().cloned().collect()
}

pub fn start_workers(store: JobStore, worker_count: usize) {
    eprintln!("[JOB QUEUE] Starting {} worker threads", worker_count);
    
    for worker_id in 0..worker_count {
        let store_clone = Arc::clone(&store);
        std::thread::spawn(move || {
            worker_loop(store_clone, worker_id);
        });
    }
}

fn worker_loop(store: JobStore, worker_id: usize) {
    eprintln!("[WORKER {}] Started", worker_id);
    
    loop {
        // Find a pending job
        let job_id = {
            let mut jobs = store.lock().unwrap();
            jobs.iter_mut()
                .find(|(_, job)| job.status == JobStatus::Pending)
                .map(|(id, job)| {
                    job.status = JobStatus::Processing;
                    id.clone()
                })
        };
        
        if let Some(job_id) = job_id {
            eprintln!("[WORKER {}] Processing job: {}", worker_id, job_id);
            
            // Get job details
            let (operation, data) = {
                let jobs = store.lock().unwrap();
                let job = jobs.get(&job_id).unwrap();
                (job.operation.clone(), job.data.clone())
            };
            
            // Process the job
            let result = match operation.as_str() {
                "json_to_toon" => {
                    match crate::converter::json_to_toon(&data) {
                        Ok(toon) => Ok(toon),
                        Err(e) => Err(format!("Conversion error: {}", e)),
                    }
                }
                "toon_to_json" => {
                    match crate::converter::toon_to_json(&data) {
                        Ok(json) => Ok(json),
                        Err(e) => Err(format!("Conversion error: {}", e)),
                    }
                }
                _ => Err(format!("Unknown operation: {}", operation)),
            };
            
            // Update job with result
            {
                let mut jobs = store.lock().unwrap();
                if let Some(job) = jobs.get_mut(&job_id) {
                    match result {
                        Ok(output) => {
                            job.status = JobStatus::Completed;
                            job.result = Some(output);
                            eprintln!("[WORKER {}] Job completed: {}", worker_id, job_id);
                        }
                        Err(error) => {
                            job.status = JobStatus::Failed;
                            job.error = Some(error.clone());
                            eprintln!("[WORKER {}] Job failed: {} - {}", worker_id, job_id, error);
                        }
                    }
                }
            }
        } else {
            // No pending jobs, sleep briefly
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
