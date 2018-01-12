use cv::*;
use cv::objdetect::CascadeClassifier;
use failure::Error;

pub fn detect_faces(image: &[u8]) -> Result<Vec<Rect>, Error> {
    let classifier = CascadeClassifier::from_path("extra/face_detect/haarcascade_frontalface_default.xml")
        .expect("Couldn't load face detection cascade");
    println!("Loading image of len {}", image.len());
    let image = Mat::imdecode(image, imgcodecs::ImreadModes::ImreadGrayscale);
    let faces = classifier.detect_with_params(&image,
        1.1,
        3,
        Size2i::new(0, 0),
        Size2i::new(100,100)
    );
    for face in faces.iter() {
        image.rectangle(face.clone());
    }
    Ok(faces)
}
