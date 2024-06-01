import * as cdk from 'aws-cdk-lib';
import * as pipeline from 'aws-cdk-lib/aws-codepipeline';
import * as pipelineactions from 'aws-cdk-lib/aws-codepipeline-actions';
import { Construct } from 'constructs';

interface PipelineStackProps extends cdk.StackProps {
  codeSourceAction: pipelineactions.CodeStarConnectionsSourceAction;
  codeBuildAction: pipelineactions.CodeBuildAction;
}

export class CodePipelineStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: PipelineStackProps) {
    super(scope, id, props);

    const { codeSourceAction, codeBuildAction } = props;

    const codePipeline = new pipeline.Pipeline(this, 'Pipeline', {
      stages: [
        {
          stageName: 'Source',
          actions: [codeSourceAction],
        },
        {
          stageName: 'Build',
          actions: [codeBuildAction],
        },
      ],
    });

    new cdk.CfnOutput(this, 'PipelineName', {
      value: codePipeline.pipelineName,
    });
  }
}
