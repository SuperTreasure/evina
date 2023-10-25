#!/bin/bash

check_workflow() {
    workflow_runs=$(curl -H "Accept: application/vnd.github.v3+json" -H "Authorization: token $INPUT_TOKEN" https://api.github.com/repos/$INPUT_OWNER/$INPUT_REPO/actions/runs?status=in_progress)
    num=$(echo $workflow_runs | jq -r .total_count)
    if [ "$num" -eq 0 ]; then
    echo "in_progress=false" >> $GITHUB_OUTPUT
    else
        for ((i=0; i<$num; i++)); do
            run_number=$(echo $workflow_runs | jq -r .workflow_runs[$i].run_number)
            name=$(echo $workflow_runs | jq -r .workflow_runs[$i].name)
            jobs_url=$(echo $workflow_runs | jq -r .workflow_runs[$i].jobs_url)
            if [[ "$name" == "$INPUT_NAME" && "$run_number" != "$INPUT_RUN_NUMBER" ]]; then
                jobs=$(curl -H "Accept: application/vnd.github.v3+json" -H "Authorization: token $INPUT_TOKEN" $jobs_url)
                num_job=$(echo $jobs | jq -r .total_count)
                echo run_number: $run_number
                echo INPUT_RUN_NUMBER: $INPUT_RUN_NUMBER
                for ((i_job=0; i_job<$num_job; i_job++)); do
                    name_job=$(echo $jobs | jq -r .jobs[$i_job].name)
                    echo name_job: $name_job
                    echo INPUT_NAME_JOB: $INPUT_NAME_JOB
                    if [[ "$name_job" == "$INPUT_NAME_JOB" ]]; then
                        steps=$(echo $jobs | jq -r .jobs[$i_job].steps[$(($INPUT_NUM_STEP - 1))])
                        name_step=$(echo $steps | jq -r .name)
                        echo name_step: $name_step
                        echo INPUT_NAME_STEP: $INPUT_NAME_STEP
                        if [[ "$name_step" == "$INPUT_NAME_STEP" ]]; then
                            if [[ "$(echo $steps | jq -r .status)" != "completed" ]]; then
                                echo t: $(echo $steps | jq -r .status)
                                echo "in_progress=true" >> $GITHUB_OUTPUT
                            else
                                echo f: $(echo $steps | jq -r .status)
                                echo "in_progress=false" >> $GITHUB_OUTPUT
                            fi
                        fi
                    fi
                done
            else
                echo "in_progress=false" >> $GITHUB_OUTPUT
            fi
        done
    fi
}

ali_upload() {
    max_retries=8  # 最大重试次数
    retries=0      # 当前重试次数

    while [[ $retries -lt $max_retries ]]; do
        # 尝试执行命令
        aliyunpan token update && aliyunpan upload 录播 / && break
    
        # 命令失败，增加重试次数
        retries=$((retries+1))
        echo "命令失败，重试次数: $retries"
        sleep 10  # 可选，等待一段时间再重试
    done

    if [[ $retries -eq $max_retries ]]; then
        # 重试次数达到上限，处理失败情况
        echo "重试次数达到上限，命令失败"
    else
        # 操作成功，处理成功情况
        echo "命令成功"
    fi
}

"$@"
