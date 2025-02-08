use crossbeam_channel::Sender;
use eframe::egui::Color32;
use linux_embedded_hal  as hal;
use std::{sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use rppal::gpio::Gpio;
use tokio::runtime::Runtime;
const A_BT: u8 = 26;
const B_BT: u8 = 19;
const C_BT: u8 = 13;
const D_BT: u8 = 06;
static I2C_DEV: &str = "/dev/i2c-1";
use hal::{Delay, I2cdev};
use bh1750::{BH1750, Resolution};

pub fn run1(
    job_run_state:Arc<Mutex<bool>>,
    start_time:Arc<Mutex<Option<Instant>>>,
    job_run_num:Arc<Mutex<u16>>,
    all_list:Arc<Mutex<Vec<(f32,u64,Color32)>>>,
    user_list:Arc<Mutex<Vec<(f32,u64,Color32)>>>,
    limit_list:Arc<Mutex<Vec<(f32,u64,Color32)>>>,
){
    thread::spawn(move||{
        let bt1 =Gpio::new().unwrap().get(A_BT).unwrap().into_input_pullup();
        let rt  = Runtime::new().unwrap();
        let mut flag = 0;
        rt.block_on(async {
            loop{
                if bt1.is_low(){
                    if flag ==0{
                        flag=1;
                        *job_run_state.lock().unwrap()=true;
                        *start_time.lock().unwrap()=Some(Instant::now());
                        
                        *job_run_num.lock().unwrap()+=1;
                        let value = (*job_run_num.lock().unwrap() as f32, 0 as u64, Color32::BLUE);
                        (*all_list.lock().unwrap()).push(value);
                        (*user_list.lock().unwrap()).push(value);
                        (*limit_list.lock().unwrap()).push(value);
                        println!("OKOKOKO");
                    }
                }else{
                    flag=0;
                }
                thread::sleep(Duration::from_millis(1));
            }
        });
    });
}

pub fn run2(
    job_run_state:Arc<Mutex<bool>>
){
    thread::spawn(move||{
        let bt2 =Gpio::new().unwrap().get(B_BT).unwrap().into_input_pullup();
        let rt  = Runtime::new().unwrap();
        let mut flag = 0;
        rt.block_on(async {
            loop{
                if bt2.is_low(){
                    if flag ==0{
                        flag=1;
                        *job_run_state.lock().unwrap()=false;
                        println!("Falseflase");
                        
                        
                    }
                }else{
                    flag=0;
                }
                thread::sleep(Duration::from_millis(1));
            }
        });
    });
}
pub fn run3(
    user_rap_state:Arc<Mutex<bool>>
){
    thread::spawn(move||{
        let bt3 =Gpio::new().unwrap().get(C_BT).unwrap().into_input_pullup();
        let rt  = Runtime::new().unwrap();
        let mut flag = 0;
        rt.block_on(async {
            loop{
                if bt3.is_low(){
                    if flag ==0{
                        flag=1;
                        thread::sleep(Duration::from_millis(50));
                        let asdasd =*user_rap_state.lock().unwrap();
                        *user_rap_state.lock().unwrap()= !asdasd;

                        println!("userrap");
                    }
                }else{
                    flag=0;
                }
                thread::sleep(Duration::from_millis(1));
            }
        });
    });
}
pub fn run4(
    all_list:Arc<Mutex<Vec<(f32,u64,Color32)>>>,
    user_list:Arc<Mutex<Vec<(f32,u64,Color32)>>>,
    limit_list:Arc<Mutex<Vec<(f32,u64,Color32)>>>,
    job_run_num:Arc<Mutex<u16>>,
    job_run_state:Arc<Mutex<bool>>,
){
    thread::spawn(move||{
        let bt =Gpio::new().unwrap().get(D_BT).unwrap().into_input_pullup();
        let rt  = Runtime::new().unwrap();
        let mut flag = 0;
        rt.block_on(async {
            loop{
                if bt.is_low(){
                    if flag ==0{
                        flag=1;
                        (*all_list.lock().unwrap()).clear();
                        (*user_list.lock().unwrap()).clear();
                        (*limit_list.lock().unwrap()).clear();
                        *job_run_num.lock().unwrap()=0;
                        *job_run_state.lock().unwrap()=false;
                    }
                }else{
                    flag=0;
                }
                thread::sleep(Duration::from_millis(1));
            }
        });
    });
}


pub fn i2c_runner(
    data_mem:Arc<Mutex<Option<f32>>>,
    sender:Sender<f32>,
    job_run_state:Arc<Mutex<bool>>,
){
    let dev = I2cdev::new(I2C_DEV).unwrap();
    let mut bh1750 = BH1750::new(dev, Delay,false);
    let rt  = Runtime::new().unwrap();
    thread::spawn(move||{
        rt.block_on(async {
            loop{
                if let Ok(lux)=bh1750.get_one_time_measurement(Resolution::High){
                    *data_mem.lock().unwrap()=Some(lux);
                    if *job_run_state.lock().unwrap(){
                        sender.send(lux).unwrap();
                    }
                }
            }
        });
    });
}