use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::Duration,
};

use orbtk::prelude::*;
use orbtk::{prelude::platform::Image, shell::WindowRequest};

static STACK_ID: &str = "STACK";

#[derive(AsAny)]
struct MainViewState {

    refresh_image_thread_handle: Option<JoinHandle<()>>,
    poll_scan_result_thread_handle: Option<JoinHandle<()>>,
    scan_result: Arc<AtomicBool>,
    current_qrcode_path: String,
    need_update_qrcode: bool,
}

impl Default for MainViewState {
    fn default() -> Self {
        Self {
            refresh_image_thread_handle: None,
            poll_scan_result_thread_handle: None,
            scan_result: Arc::new(AtomicBool::new(false)),
            current_qrcode_path: get_new_qrcode(0),

            need_update_qrcode: true,
        }
    }
}

enum Message {
    UpdatetQRCodeImage(String),
    ScanCodeFinished,
}

fn get_new_qrcode(i:i32)->String{
    return format!("./img/qrcode{}.png",i);
}

fn init_refresh_qrcode(state: &mut MainViewState, ctx: &mut Context) {
    let msg_adapter = ctx.message_adapter();
    let entity = ctx.widget().entity();
    let scan_code_result_clone = state.scan_result.clone();
    let handle = std::thread::spawn(move || {
        let mut counter: i32 = 0;
        loop {
            println!("waiting for refresh qrcode");
            counter += 1;
            std::thread::sleep(Duration::from_secs(3));
            if scan_code_result_clone.load(Ordering::SeqCst) {
                break;
            }
            
            let new_p =get_new_qrcode(counter);

            msg_adapter.send_message(
                Message::UpdatetQRCodeImage(new_p),
                entity,
            );
            println!("new qrcode in hand");
        }
        println!("quit refresh_qrcode thread!");
    });
    state.refresh_image_thread_handle = Some(handle);
}

fn get_user_scan_code_result(i:i32)->bool{
    match i {
        0..=3 => false,
        _ => true,
    }
}
fn init_poll_scan_result(state: &mut MainViewState, ctx: &mut Context) {
    let msg_adapter = ctx.message_adapter();
    let entity = ctx.widget().entity();
    let handle = std::thread::spawn(move || {
        let mut count: i32 = 0;
        loop {
            count += 1;

            std::thread::sleep(Duration::from_secs(3));

            //
            let r = get_user_scan_code_result(count);
            if r{
                msg_adapter.send_message(Message::ScanCodeFinished, entity);
                println!("user scan code success !");
                break;
            }
        }
        println!("quit poll_scan_result thread!");
    });
    state.poll_scan_result_thread_handle = Some(handle);
}
impl State for MainViewState {
    fn init(&mut self, _registry: &mut Registry, _ctx: &mut Context) {
        println!("int init.");
        init_refresh_qrcode(self, _ctx);
        init_poll_scan_result(self, _ctx);
    }

    fn cleanup(&mut self, _registry: &mut Registry, _ctx: &mut Context) {
        println!("in cleanup1");
        let tmp = self.refresh_image_thread_handle.take();
        if let Some(h) = tmp {
            let _ = h.join();
            println!("in cleanup2");
        }
        let tmp = self.poll_scan_result_thread_handle.take();
        if let Some(h) = tmp {
            let _ = h.join();
            println!("in cleanup3");
        }
        println!("in cleanup4");
    }

    fn update(&mut self, _: &mut Registry, ctx: &mut Context) {
        if self.need_update_qrcode {
            let old_img_widget = ctx.entity_of_child("myimg").unwrap();
            let stack = ctx.entity_of_child(STACK_ID).unwrap();
            ctx.remove_child(old_img_widget);

            let mut build_ctx = ctx.build_context();

            let new_img_widget = create_image_widget(&mut build_ctx, &self.current_qrcode_path);
            ctx.append_child_entity_to(new_img_widget, stack);

            self.need_update_qrcode = false;
            println!("in update : qrcode image widget updated");
        }
    }


    fn messages(
        &mut self,
        mut messages: MessageReader,
        _registry: &mut Registry,
        ctx: &mut Context,
    ) {
        println!("in messages:");

        for message in messages.read::<Message>() {
            match message {
                Message::UpdatetQRCodeImage(p) => {
                    println!("in messages: UpdateImage");
                    self.current_qrcode_path = p;
                    self.need_update_qrcode= true;

                    ctx.window().update_dirty(true);
                }
                Message::ScanCodeFinished => {
                    println!("in messages: ScanCodeFinished");
                    self.scan_result.store(true, Ordering::SeqCst);
                    ctx.send_window_request(WindowRequest::Close);
                    //ctx.window().update(true);
                    //ctx.event_adapter().push_event_direct(ctx.widget().entity(), SystemEvent::Quit);
                }
            }
        }
    }
}

fn create_image_widget(bc: &mut BuildContext, img_path: &str) -> Entity {
    ImageWidget::new()
        .id("myimg")
        .v_align("center")
        .h_align("center")
        .image(Image::from_path(img_path).unwrap())
        .width(245.0)
        .height(245.0)
        .build(bc)
}


widget!(MainView<MainViewState>);

impl Template for MainView {
    fn template(self, id: Entity, bc: &mut BuildContext) -> Self {
        println!("in template");
        self.name("MainView").margin(16.0).child(
            Stack::new()
                .id(STACK_ID)
                .h_align("center")
                .v_align("center")
                .child(create_image_widget(bc, &get_new_qrcode(0)))
                .build(bc),
        )
    }
}

fn gui() {
    // use this only if you want to run it as web application.
    //orbtk::initialize();

    let app = Application::new().window(|ctx| {
        Window::new()
            .title("scan qrcode login")
            .position((100.0, 100.0))
            .size(245.0, 245.0)
            .child(MainView::new().build(ctx))
            .build(ctx)
    });

    app.run();
}

fn other() {
    println!("****my other code run here***!");
}

fn main() {
    gui();
    other();
}
