use failure::Error;
use std::process::{Command, Stdio};
use std::io::{Read,Write,BufRead,BufWriter};
use std::path::PathBuf;

/* available characters: 0123456789+-÷×
   length of each line is 8, the first being a math operator, followed by 7 digits
   there are 8 lines per image - therefore, there's a total of 64 glyphs/characters per image

   page segmentation mode is 11, aka "Sparse text", but 4 may also work("Assume a single column of text of variable sizes")

   used fonts:
   Aaaiight! https://www.fontzillion.com/fonts/jw-type/aaaiight?utm_source=fontsquirrel.com&utm_medium=matcherator_link&utm_campaign=aaaiight
   Arial
*/

pub fn image_to_text(image: &[u8]) -> Result<String, Error> {
    let tess = Command::new("tesseract")
        .args(&["stdin", "stdout", "--oem", "0", "--psm", "11", "-l", "eng",
                "-c", "tessedit_char_whitelist=0123456789+-÷×", "stdout"])
        .env("TESSDATA_PREFIX", "./extra/visual_basic_math/v3_traineddata")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    tess.stdin.unwrap().write_all(image)?;
    let mut string = String::new();
    tess.stdout.unwrap().read_to_string(&mut string)?;
    string = string.replace(" ","");
    let lines = string
        .lines()
        .filter(|l| l.trim().len() > 0)
        .collect::<Vec<&str>>();
    let string = lines.join("\n");
    println!("OCR result is:\n{}",string);
    Ok(string)
}
