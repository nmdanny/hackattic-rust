$tessdata='.\tessdata_fast'
$langdata='.\langdata'
$traindata='.\v4traindata'
$evaldata='.\v4evaldata'
$trained_out='.\trained'
echo "Beginning fine-tune training of eng tessdata with new fonts and math symbols"

<# Requires running ./generate_datas.sh to satisfy the following params:
     continue_from
     traineddata
     train_listfile

   Requires the following datafile:
     tessdata\eng.traineddata
#>

<#
lstmtraining.exe `
    --model_output $traindata\model `
    --continue_from $traindata\eng.lstm `
    --traineddata $traindata\eng\eng.traineddata `
    --train_listfile $traindata\eng.training_files.txt `
    --max_iterations 400  
#>

mkdir $traindata\output\
lstmtraining.exe `
    --model_output $traindata\output\ `
    --continue_from $traindata\eng.lstm `
    --traineddata $traindata\eng\eng.traineddata `
    --train_listfile $traindata\eng.training_files.txt `
    --old_traineddata $tessdata\eng.traineddata `
    --max_iterations 3600  

mkdir $trained_out
lstmtraining.exe --stop_training `
  --continue_from $traindata\output\_checkpoint `
  --traineddata $traindata\eng\eng.traineddata `
  --model_output $trained_out\eng.traineddata