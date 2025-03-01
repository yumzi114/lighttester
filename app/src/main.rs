
mod component;
mod appthreads;
use std::{fmt::format, sync::{Arc, Mutex}, time::Instant};

use appthreads::{i2c_runner, run1, run2, run3, run4};
use component::setup_custom_fonts;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui::{self, vec2, Align, CentralPanel, Checkbox, Color32, RichText, Vec2, ViewportBuilder};

use egui_extras::{TableBuilder, Column};


#[tokio::main]
async fn main() {
    let windows = ViewportBuilder{
        // title: Some(String::from("Chorusing App")),
        // app_id: Some(String::from("Chorusing App")),
        // fullsize_content_view: Some(true),
        titlebar_shown: Some(true),
        resizable: Some(false),
        fullscreen:Some(true),
        ..Default::default()
    };
    let options = eframe::NativeOptions {
        viewport:windows,
        // default_theme:Theme::Dark,
        ..Default::default()
    };
    // let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App", 
        options, 
        Box::new(|cc| 
            Ok(Box::new({
                let app = MyEguiApp::new(cc);
                i2c_runner(app.i2c_data.clone(),app.app_sender.clone(),app.job_run_state.clone());
                run1(app.job_run_state.clone(),app.start_time.clone(),app.job_run_num.clone(),app.all_list.clone(),app.user_list.clone(),app.limit_list.clone());
                run2(app.job_run_state.clone());
                run3(app.user_rap_state.clone());
                run4(app.all_list.clone(),app.user_list.clone(),app.limit_list.clone(),app.job_run_num.clone(),app.job_run_state.clone());
                app
            }
                
            ))));
}

// #[derive(Default)]
struct MyEguiApp {
    job_run_state:Arc<Mutex<bool>>,
    job_run_num:Arc<Mutex<u16>>,
    user_rap_state:Arc<Mutex<bool>>,
    i2c_data:Arc<Mutex<Option<f32>>>,
    app_sender:Sender<f32>, 
    app_receiver:Receiver<f32>,
    // all_list:Vec<(f32,u64,Color32)>,
    all_list:Arc<Mutex<Vec<(f32,u64,Color32)>>>,
    user_list:Arc<Mutex<Vec<(f32,u64,Color32)>>>,
    limit_list:Arc<Mutex<Vec<(f32,u64,Color32)>>>,
    flate_data:f32,
    limit_data:f32,
    start_time:Arc<Mutex<Option<Instant>>>,
    user_rap:bool,
    size_rap:bool,
    auto_rap:bool,
    dep_value:f32,
    time_fmt:bool,
    graph_view:bool,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let (app_sender, app_receiver) = unbounded::<f32>();
        // let now: Instant = Instant::now();
        Self { 
            app_sender,
            app_receiver,
            i2c_data:Arc::new(Mutex::new(None)),
            job_run_state: Arc::new(Mutex::new(false)),
            user_rap_state: Arc::new(Mutex::new(false)),
            all_list:Arc::new(Mutex::new(vec![])),
            user_list:Arc::new(Mutex::new(vec![])),
            limit_list:Arc::new(Mutex::new(vec![])),
            flate_data:0.,
            limit_data:0.,
            job_run_num:Arc::new(Mutex::new(0)),
            start_time:Arc::new(Mutex::new(None)),
            user_rap:true,
            size_rap:true,
            auto_rap:true,
            dep_value:0.,
            time_fmt:false,
            graph_view:false,
            // ..Default::default()
        }
    }
}

impl eframe::App for MyEguiApp {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    ctx.request_repaint();
    
    if let Ok(data)=self.app_receiver.try_recv(){
        
        if let Some(now)=*self.start_time.lock().unwrap(){
            let time = now.elapsed().as_secs();
            let color = if self.flate_data > data {
                Color32::RED
            }else if self.flate_data < data{
                Color32::GREEN
            }else {Color32::WHITE};
            let user_color = if *self.user_rap_state.lock().unwrap(){
                    Color32::GREEN
                }else{
                    Color32::WHITE
                };
            if self.auto_rap{
                // let guard = ;
                (*self.all_list.lock().unwrap()).push((data,time,color));
                // *guard.push((data,time,color));
                // self.all_list.push((data,time,color));
                self.flate_data=data;
            }
            if self.user_rap{
                
                (*self.user_list.lock().unwrap()).push((data,time,user_color));
            }
            if self.dep_value!=0.{
                let limit_color;
                if self.dep_value <=data-self.limit_data{
                    limit_color=Color32::GREEN;
                    self.limit_data=data;
                    (*self.limit_list.lock().unwrap()).push((data,time,limit_color));
                }else if self.dep_value <=self.limit_data -data{
                    limit_color=Color32::RED;
                    self.limit_data=data;
                    (*self.limit_list.lock().unwrap()).push((data,time,limit_color));
                }else {
                    // limit_color=Color32::WHITE;
                    // self.limit_data=data;
                }
                
                // if (data - self.limit_data).abs()>=self.dep_value{

                // }
            }
            
        }
    }
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui|{
            ui.add_space(20.);
            ui.vertical(|ui|{
                ui.horizontal(|ui|{
                    if let Some(data) = *self.i2c_data.lock().unwrap(){
                        let title = format!("센서 : {:.2}",data);
                        ui.add_sized([700.,100.], egui::widgets::Label::new(RichText::new(title).strong().size(80.0)));
                        // ui.label(RichText::new(title).strong().size(80.0));
                        // println!("Lux = {:.2}",lux);
                    }
                    ui.add_space(20.);
                    match self.time_fmt {
                        true=>{
                            ui.label(RichText::new("시간형식").strong().size(60.0));
                        },
                        false=>{
                            ui.label(RichText::new("초단위").strong().size(60.0));
                        },
                    }
                    ui.add(toggle(&mut self.time_fmt));
                    ui.add_space(20.);
                    match self.graph_view {
                        true=>{
                            ui.label(RichText::new("그래프").strong().size(60.0));
                        },
                        false=>{
                            ui.label(RichText::new("데이터").strong().size(60.0));
                        },
                    }
                    
                    ui.add(toggle(&mut self.graph_view));
                });
                
            });
            
            
        });
        match self.graph_view {
            true=>{

            },
            false=>{
                egui::CentralPanel::default().show(ctx, |ui| {
                    TableBuilder::new(ui)
                    .cell_layout(egui::Layout::top_down(egui::Align::Center))
                    // .column(Column::auto().resizable(true))
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .header(15.0, |mut header| {
                        header.col(|ui| {
                            if ui.heading(RichText::new("사용자버튼").strong().size(50.0).color(if self.user_rap{Color32::GREEN}else{Color32::WHITE})).clicked(){
                                self.user_rap=!self.user_rap;
                            };
                        });
                        header.col(|ui| {
                            ui.vertical_centered(|ui|{
                                if ui.heading(RichText::new("범위측정").strong().size(50.0).color(if self.dep_value>0.0{Color32::GREEN}else{Color32::WHITE})).clicked(){
                                    self.size_rap=!self.size_rap;
                                };
                                ui.add_sized([50.,40.],egui::Slider::new(&mut self.dep_value, 0.0..=1000.0).suffix(""));
                                
                                // ui.add_sized([40.0, 20.0], egui::DragValue::new(&mut self.dep_value));
                                
                                // ui.add_sized([100.,100.],egui::Slider::new(&mut self.dep_value, 0.0..=1000.0).suffix(""));
                                // ui.label("text");
                            });
                            
                        });
                        header.col(|ui| {
                            if ui.heading(RichText::new("자동측정").strong().size(50.0).color(if self.auto_rap{Color32::GREEN}else{Color32::WHITE})).clicked(){
                                self.auto_rap=!self.auto_rap;
                            };
                            
                        });
                    })
                    .body(|mut body| {
                        body.row(700.0, |mut row| {
                            row.col(|ui| {
                                ui.add_space(20.);
                                ui.push_id(1, |ui|{
                                    egui::ScrollArea::vertical()
                                    // .min_scrolled_width(width)
                                    .show(ui, |ui| {
                                        for (data, time,color) in (*self.user_list.lock().unwrap()).iter(){
                                            // let time = time.to_string();
                                            // let trimmed_time = &time[..time.len()];
                                            let fmt  = 
                                            match self.time_fmt {
                                                false=>format!("{:.2} - {} (S)",data,time),
                                                true=>{
                                                    let (h, m, s) = seconds_to_hms(*time);
                                                    format!("{:.2} - {}:{}:{} (T)",data,h,m,s)
                                                },
                                            };
                                            
                                            
                                            ui.label(RichText::new(fmt).strong().size(30.0).color(*color));
                                            let rect = ui.available_rect_before_wrap();
                                            if *self.job_run_state.lock().unwrap(){
                                                ui.scroll_to_rect(rect, Some(Align::BOTTOM));
                                            }
                                            
                                        }
                                    });
                                });
                            });
                            row.col(|ui| {
                                ui.push_id(2, |ui|{
                                    ui.add_space(20.);
                                    egui::ScrollArea::vertical()
                                    // .min_scrolled_width(width)
                                    .show(ui, |ui| {
                                        for (data, time,color) in (*self.limit_list.lock().unwrap()).iter(){
                                            // let time = time.to_string();
                                            // let trimmed_time = &time[..time.len()];
                                            let fmt  = 
                                            match self.time_fmt {
                                                false=>format!("{:.2} - {} (S)",data,time),
                                                true=>{
                                                    let (h, m, s) = seconds_to_hms(*time);
                                                    format!("{:.2} - {}:{}:{} (T)",data,h,m,s)
                                                },
                                            };
                                            
                                            ui.label(RichText::new(fmt).strong().size(30.0).color(*color));
                                            let rect = ui.available_rect_before_wrap();
                                            if *self.job_run_state.lock().unwrap(){
                                                ui.scroll_to_rect(rect, Some(Align::BOTTOM));
                                            }
                                            // ui.scroll_to_rect(rect, Some(Align::BOTTOM));
                                        }
                                    });
                                });
                            });
                            row.col(|ui| {
                                ui.push_id(3, |ui|{
                                    ui.add_space(20.);
                                    egui::ScrollArea::vertical()
                                    // .min_scrolled_width(width)
                                    .show(ui, |ui| {
                                        for (data, time,color) in (*self.all_list.lock().unwrap()).iter(){
                                            // let time = time.to_string();
                                            // let trimmed_time = &time[..time.len()];
                                            let fmt  = 
                                            match self.time_fmt {
                                                false=>format!("{:.2} - {} (S)",data,time),
                                                true=>{
                                                    let (h, m, s) = seconds_to_hms(*time);
                                                    format!("{:.2} - {} :{} :{} (T)",data,h,m,s)
                                                },
                                            };
                                            
                                            ui.label(RichText::new(fmt).strong().size(30.0).color(*color));
                                            let rect = ui.available_rect_before_wrap();
                                            if *self.job_run_state.lock().unwrap(){
                                                ui.scroll_to_rect(rect, Some(Align::BOTTOM));
                                            }
                                            // ui.scroll_to_rect(rect, Some(Align::BOTTOM));
                                        }
                                    });
                                });
                            });
                        });
                    });
                });
            }
        }
        
        egui::TopBottomPanel::bottom("my_panel1").show(ctx, |ui|{
            ui.add_space(20.);
            ui.vertical(|ui|{
                match *self.job_run_state.lock().unwrap() {
                    false=>{
                        ui.label(RichText::new("STOP").strong().size(50.0).color(Color32::RED));
                    },
                    true=>{
                        let fmt = format!("Running -{}-",* self.job_run_num.lock().unwrap());
                        ui.label(RichText::new(fmt).strong().size(50.0).color(Color32::GREEN));
                    }
                }
                // ui.horizontal(|ui|{
                //     if let Some(data) = *self.i2c_data.lock().unwrap(){
                //         let title = format!("센서 : {:.2}",data);
                //         ui.label(RichText::new(title).strong().size(80.0));
                //         // println!("Lux = {:.2}",lux);
                //     }
                    
                // });
            });
            
        });
       
   }
}


pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(4.0, 2.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed(); // report back that the value changed
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter().rect(
            rect,
            radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            // egui::StrokeKind::Inside,
        );
        // Paint the circle, animating it from left to right with `how_on`:
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }
    response
}


pub fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| toggle_ui(ui, on)
}

fn seconds_to_hms(seconds: u64) -> (u64, u64, u64) {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    (hours, minutes, secs)
}