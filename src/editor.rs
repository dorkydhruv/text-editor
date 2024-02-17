
use crate::Row;
use crate::Document;
use crate::terminal::Terminal;
use termion::event::Key;
use std::env;
const VERSION:&str =env!("CARGO_PKG_VERSION");
pub struct Editor{
    should_quit: bool,
    terminal: Terminal,
    position: Position,
    document: Document,
}

#[derive(Default)]
pub struct Position{
    pub x:usize,
    pub y:usize,
}
impl Editor{
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
            Terminal::cursor_position(&self.position);
        }
        Terminal::flush()
    }
    pub fn default() -> Self{
        let args: Vec<String>=env::args().collect();
        let document=if args.len()>1{
            let filename = &args[1];
            Document::open(&filename).unwrap_or_default()
        }else{
            Document::default()
        };
        Self { should_quit: false, 
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            position: Position::default(),
            document: document
        }

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
    fn draw_rows(&self){
        let height  = self.terminal.size().height;
        for terminal_row in 0..height-1{
            Terminal::clear_current_line();
            if let Some(row)=self.document.row(terminal_row as usize){
                self.draw_row(row);
            }else if terminal_row==height/3 && self.document.is_empty() {
                self.draw_welcome_message();

            }
            else{
                println!("~\r");
            
            }
        }
    }

    fn process_keypress(&mut self) -> Result<(),std::io::Error>{
        let pressed_key = Terminal::read_key()?;
        match pressed_key{
            Key::Ctrl('q')=> self.should_quit=true,
            Key::Up | Key::Down | Key::Left | Key::Right | Key::PageDown | Key::PageUp | Key::End | Key::Home => self.move_cursor(pressed_key),
            _ => (),
        }
        Ok(())
    }

    pub fn draw_row(&self,row:&Row){
        let start = 0;
        let end = self.terminal.size().width as usize;
        let row = row.render(start,end);
        println!("{}\r",row);
    }

    fn move_cursor(&mut self,key: Key){
        let Position{mut x,mut y}=self.position;
        let size = self.terminal.size();
        let h = size.height as usize;
        let w = size.width as usize;
        match key{
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y<h-1{
                    y=y.saturating_add(1);
                }
            },
            Key::Left => x = x.saturating_sub(1),
            Key::Right => {
                    if x<w {
                        x = x.saturating_add(1);
                    }
                },
            Key::PageUp => y=0,
            Key::PageDown => y = h-1,
            Key::Home => x=0,
            Key::End => x = w,
            _ => (),
        
        }
        self.position = Position{x,y};
    }
}



fn die(e: &std::io::Error) -> () {
    Terminal::clear_screen();
    panic!("{e:?}");
    // io::stdout().flush().unwrap()
}