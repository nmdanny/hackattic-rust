use failure::Error;
use std::process::Command;
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

/* command to run:
tesseract stdin --psm 11 -l eng stdout --tessdata-dir=TESSDATA_PATH_HERE
TESSDATA_PATH_HERE should contain an eng.traineddata file which supports above fonts and characters
*/

pub fn image_to_text(image: &[u8]) -> Result<String, Error> {
    unimplemented!()
}
