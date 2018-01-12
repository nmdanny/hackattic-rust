use super::HackatticChallenge;
use reqwest;
use failure::Error;
use std::io::Read;
use cv::Rect;

pub mod detection;

#[derive(Debug, Deserialize)]
pub struct Problem {
    image_url: String
}

#[derive(Debug, Serialize)]
pub struct Solution {
    face_tiles: Vec<(usize, usize)>
}

fn face_rect_to_usize(rect: &Rect) -> (usize, usize) {
    // each picture is 800x800 with 8x8 tiles, each tile being 100x100
    let row = (rect.x  / 100) as usize;
    let col = (rect.y / 100) as usize;
    (row,col)
}

pub struct FaceDetection;
impl HackatticChallenge for FaceDetection {
    type Problem = Problem;
    type Solution = Solution;

    fn make_solution(problem: &Self::Problem) -> Result<Self::Solution, Error> {
        let image_buf = {
            let mut res = reqwest::get(&problem.image_url)?;
            let mut buf = Vec::new();
            res.read_to_end(&mut buf)?;
            buf
        };
        let face_recs = detection::detect_faces(&image_buf)?;
        Ok(Solution {
            face_tiles: face_recs.iter().map(face_rect_to_usize).collect()
        })
    }

    fn challenge_name() -> &'static str { "basic_face_detection" }
}