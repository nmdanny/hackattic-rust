#!/bin/bash

fonts='C:/Windows/Fonts'
tessdata='./tessdata_best' # this needs to be best, or lstmtraining will crash
langdata='./langdata'
traindata='./v4traindata'
evaldata='./v4evaldata'
<<EVAL
echo "Generating eval data"

tesstrain.sh \
  --fonts_dir $fonts \
  --lang eng --linedata_only \
  --noextract_font_properties \
  --training_text ./training_text_with_ops.txt \
  --tessdata_dir $tessdata \
  --langdata_dir $langdata \
  --fontlist "Arial" "aaaiight!" "aaaiight! fat" \
  --output_dir $evaldata
EVAL

mkdir $traindata
echo "Generating eng.training_files and eng.traineddata at $traindata"
tesstrain.sh \
  --fonts_dir $fonts \
  --noextract_font_properties \
  --lang eng --linedata_only \
  --training_text ./training_text_with_ops.txt \
  --tessdata_dir $tessdata \
  --langdata_dir $langdata \
  --output_dir $traindata


echo "Creating eng.lstm from tessdata"

combine_tessdata.exe -e $tessdata/eng.traineddata $traindata/eng.lstm

#  --fontlist "Arial" "aaaiight!" "aaaiight! fat" \
