use std::cmp;
use unicode_segmentation::UnicodeSegmentation;
#[derive(Default)]
pub struct Row{
    pub string: String,
    len:usize
}

impl From<&str> for Row{
    fn from(s:&str)->Self{
        let mut row = Self{
            string :String::from(s),
            len:0,
        };
        row.update_len();
        row
    }
}
impl Row{
    pub fn render(&self,start:usize,end:usize)->String{
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut res = String::new();
        for grapheme in self.string[..].graphemes(true).skip(start).take(end-start){
            if grapheme == "\t"{
                res.push_str(" ");

            }else{
                res.push_str(grapheme);
            }
        }
        res
    }
    pub fn len(&self)->usize{
        self.len
    }
    pub fn is_empty(&self)->bool{
        self.len ==0
    }
    fn update_len(&mut self){
        self.len = self.string.graphemes(true).count();
    }
    pub fn insert(&mut self,at:usize,c:char){
        if at >=self.len(){
            self.string.push(c);
        }else{
            let mut result:String = self.string[..].graphemes(true).take(at).collect();
            let remainder:String = self.string[..].graphemes(true).skip(at).collect();
            result.push(c);
            result.push_str(&remainder);
            self.string = result;
        }
        self.update_len();
    }
    pub fn delete(&mut self,at:usize){
        if at>=self.len(){
            return;
        }
        let mut result:String = self.string[..].graphemes(true).take(at).collect();
        let rem :String = self.string[..].graphemes(true).skip(at+1).collect();
        result.push_str(&rem);
        self.string = result;
        self.update_len();
    }
    pub fn append(&mut self,new:&Self){
        self.string = format!("{}{}",self.string,new.string);
        self.update_len();
    }
    pub fn split(&mut self,at:usize)->Self{
        let beginning: String = self.string[..].graphemes(true).take(at).collect();
        let remainder :String = self.string[..].graphemes(true).skip(at).collect();
        self.string =beginning;
        self.update_len();
        Self::from(&remainder[..])
    }
}