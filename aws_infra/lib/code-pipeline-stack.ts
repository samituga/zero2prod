import * as cdk from 'aws-cdk-lib';
import * as codedeploy from 'aws-cdk-lib/aws-codedeploy';
import * as pipeline from 'aws-cdk-lib/aws-codepipeline';
import * as pipelineactions from 'aws-cdk-lib/aws-codepipeline-actions';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import { Construct } from 'constructs';

interface PipelineStackProps extends cdk.StackProps {
  fargateService: ecs.FargateService;
  deploymentGroup: codedeploy.EcsDeploymentGroup;
  repository: ecr.Repository;
  connectionArn: string;
}

export class PipelineStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: PipelineStackProps) {
    super(scope, id, props);

    const sourceOutput = new pipeline.Artifact();
    const buildOutput = new pipeline.Artifact();

    const codePipeline = new pipeline.Pipeline(this, 'Pipeline', {
      pipelineName: 'MyPipeline',
      stages: [
        {
          stageName: 'Source',
          actions: [
            new pipelineactions.CodeStarConnectionsSourceAction({
              actionName: 'CodeStarSource',
              owner: 'your-github-username',
              repo: 'your-repo-name',
              branch: 'main',
              connectionArn: props.connectionArn,
              output: sourceOutput,
            }),
          ],
        },
        {
          // TODO code-build-stack.ts
          stageName: 'Build',
          actions: [
            new pipelineactions.CodeBuildAction({
              actionName: 'Build',
              project: new cdk.aws_codebuild.PipelineProject(this, 'CodeBuildProject', {
                environment: {
                  buildImage: cdk.aws_codebuild.LinuxBuildImage.STANDARD_5_0,
                  privileged: true,
                },
                buildSpec: cdk.aws_codebuild.BuildSpec.fromSourceFilename('buildspec.yaml'),
              }),
              input: sourceOutput,
              outputs: [buildOutput],
            }),
          ],
        },
        {
          stageName: 'Deploy',
          actions: [
            new pipelineactions.CodeDeployEcsDeployAction({
              actionName: 'ECS_Deploy',
              deploymentGroup: props.deploymentGroup,
              appSpecTemplateFile: buildOutput.atPath('appspec.yaml'),
              taskDefinitionTemplateFile: buildOutput.atPath('taskdef.json'), //TODO generate this file from FargateTaskDefinition toString
              containerImageInputs: [{ input: buildOutput }],
            }),
          ],
        },
      ],
    });

    new cdk.CfnOutput(this, 'PipelineName', {
      value: codePipeline.pipelineName,
    });
  }
}
