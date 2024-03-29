use crate::Row;
use crate::Document;
use crate::terminal::Terminal;
use termion::color;
use termion::event::Key;
use std::env;
use std::time::Duration;
use std::time::Instant;
const STATUS_FG_COLOR: color::Rgb = color::Rgb(63,63,63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239,239,239);
const VERSION:&str =env!("CARGO_PKG_VERSION");
const QUIT_TIMES:u8 = 3;
pub struct Editor{
    should_quit: bool,
    terminal: Terminal,
    position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times:u8,
}

#[derive(Default)]
pub struct Position{
    pub x:usize,
    pub y:usize,
}

struct StatusMessage{
    text: String,
    time:Instant,
}

impl StatusMessage{
    fn from(message:String)->Self{
        Self{
            text: message,
            time: Instant::now(),
        }
    }
}

impl Editor{
    pub fn default() -> Self{
        let args: Vec<String>=env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-S to save | Ctrl-Q to quit.");
        let document=if args.len()>1{
            let filename = &args[1];
            let doc =Document::open(&filename);
            if doc.is_ok(){
                doc.unwrap()
            }else{
                initial_status = format!("ERR: Could not open file {}",filename);
                Document::default()
            }
        }else{
            Document::default()
        };
        Self { should_quit: false, 
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            position: Position::default(),
            offset: Position::default(),
            document,
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
        }

    }
    pub fn run(&mut self){
        loop{
            if let Err(error) = self.refresh_screen(){
                die(&error);
                
            }
            if self.should_quit{
                
                // Terminal::clear_screen();
                // println!("Goodbye.\r");
                std::process::exit(0);
                // break;/
            }
            if let Err(error) = self.process_keypress(){
                die(&error);
            }
        }
    }
    fn refresh_screen(&self)-> Result<(),std::io::Error>{
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit{
            Terminal::clear_screen();
            println!("Goodbye.\r");
        }else{
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&self.position);
            Terminal::cursor_position(&Position{
                x: self.position.x.saturating_sub(self.offset.x),
                y: self.position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }
    
    fn draw_welcome_message(&self){
        let mut welcome_message = format!("Terminal Editor --- version {:?}",VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len)/2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}",spaces,welcome_message);
        welcome_message.truncate(width);
        println!("{}\r",welcome_message);
    }
    fn draw_status_bar(&self){
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty(){
            " (modified)"
        }else{
            ""
        };
        let mut file_name = "[No Name]".to_string();
        if let Some(name)= &self.document.filename{
            file_name  = name.clone();
            file_name.truncate(20);
        }
        status = format!("{} - {} lines{}",file_name,self.document.len(),modified_indicator);
        let line_indicator = format!("{}/{}",self.position.y.saturating_add(1),self.document.len());
        let len  = status.len() + line_indicator.len();
        if width >len{
            status.push_str(&" ".repeat(width-len));
        }
        status = format!("{}{}",status,line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r",status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn draw_message_bar(&self){
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time <Duration::new(5, 0){
            let mut txt = message.text.clone();
            txt.truncate(self.terminal.size().width as usize);
            print!("{}",txt);
        }
    }
    fn draw_rows(&self){
        let height  = self.terminal.size().height;
        for terminal_row in 0..height{
            Terminal::clear_current_line();
            if let Some(row)=self.document.row(terminal_row as usize+ self.offset.y){
                self.draw_row(row);
            }else if terminal_row==height/3 && self.document.is_empty() {
                self.draw_welcome_message();

            }
            else{
                println!("~\r");
            
            }
        }
    }

    fn prompt(&mut self,prompt:&str)->Result<Option<String>,std::io::Error>{
        let mut result = String::new();
        loop{
            self.status_message = StatusMessage::from(format!("{}{}",prompt,result));
            self.refresh_screen()?;
            match Terminal::read_key()? {
                Key::Backspace=>{
                    if !result.is_empty(){
                        result.truncate(result.len()-1);
                    }
                },
                Key::Char('\n')=>break,
                Key::Char(c)=>{
                    if !c.is_control(){
                        result.push(c);
                    }
                },
                Key::Esc=>{
                    result.truncate(0);
                    break;
                },
                _=>(),

            }
        }
        self.status_message =StatusMessage::from(String::new());
        if result.is_empty(){
            return Ok(None);
        }
        Ok(Some(result))
    }

    fn save(&mut self){
        if self.document.filename.is_none(){
            let new_name = self.prompt("Save as:").unwrap_or(None);
            if new_name.is_none(){
                self.status_message = StatusMessage::from("Save aborted".to_string());
                return;
            }
            self.document.filename =new_name;
        }
        if self.document.save().is_ok(){
            self.status_message = StatusMessage::from("File saved successfully".to_string());
        }else{
            self.status_message = StatusMessage::from("Error saving file".to_string());
        }
    }

    fn process_keypress(&mut self) -> Result<(),std::io::Error>{
        let pressed_key = Terminal::read_key()?;
        match pressed_key{
            Key::Ctrl('q')=> {
                if self.quit_times>0 && self.document.is_dirty(){
                    self.status_message = StatusMessage::from(format!(
                        "WARNING!!! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                        self.quit_times
                    ));
                    self.quit_times-=1;
                    return Ok(());
                }
                self.should_quit=true;
            }
            Key::Ctrl('s')=>self.save(),
            Key::Char(c)=>{
                self.document.insert(&self.position, c);
                self.move_cursor(Key::Right);
            },
            Key::Delete=>self.document.delete(&self.position),
            Key::Backspace=>{
                if self.position.x>0 ||self.position.y>0{
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.position);
                }
            },
            Key::Up | Key::Down | Key::Left | Key::Right | Key::PageDown | Key::PageUp | Key::End | Key::Home => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        if self.quit_times<QUIT_TIMES{
            self.quit_times=QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    pub fn draw_row(&self,row:&Row){
        let width = self.terminal.size().width as usize;

        let start = self.offset.x;
        let end = self.offset.x+width;
        let row = row.render(start,end);
        println!("{}\r",row);
    }

    fn scroll(&mut self){
        let Position{x,y} = self.position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let  offset = &mut self.offset;
        if y<offset.y{
            offset.y=y;
        }else if y>=offset.y.saturating_add(height){
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x<offset.x{
            offset.x=x;
        }else if x>=offset.x.saturating_add(width){
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self,key: Key){
        let Position{mut x,mut y}=self.position;
        let terminal_height =  self.terminal.size().height as usize;
        let h = self.document.len();
        let mut w = if let Some(row)=self.document.row(y){
            row.len()
        }else{
            0
        };
        match key{
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y<h-1{
                    y=y.saturating_add(1);
                }
            },
            Key::Left => {
                if x>0{
                    x-=1;
                }else if y>0{
                    y-=1;
                    if let Some(row) = self.document.row(y){
                        x=row.len();
                    }else{
                        x=0;
                    }
                }
            },
            Key::Right => {
                    if x<w {
                        x +=1;
                    }else if y<h{
                        y+=1;
                        x=0;
                    }
                },
            Key::PageUp => {
                y = if y>terminal_height{
                    y - terminal_height
                }else{
                    0
                }

            },
            Key::PageDown =>{
                y = if y.saturating_add(terminal_height)<h{
                    y.saturating_add(terminal_height)
                }else{
                    h-1
                }
            },
            Key::Home => x=0,
            Key::End => x = w,
            _ => (),
        
        }
        w = if let Some(row) = self.document.row(y){
            row.len()
        }else{
            0
        };
        if x>w{
            x=w;
        }
        self.position = Position{x,y};
    }
}



fn die(e: &std::io::Error) -> () {
    Terminal::clear_screen();
    panic!("{e:?}");
    // io::stdout().flush().unwrap()
}