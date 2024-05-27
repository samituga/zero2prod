import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { Artifact } from 'aws-cdk-lib/aws-codepipeline';
import { Pipeline } from 'aws-cdk-lib/aws-codepipeline';
import { CodeBuildAction, CodeStarConnectionsSourceAction, CodeDeployEcsDeployAction } from 'aws-cdk-lib/aws-codepipeline-actions';
import { FargateService } from 'aws-cdk-lib/aws-ecs';
import { EcsDeploymentGroup } from 'aws-cdk-lib/aws-codedeploy';
import { Repository } from 'aws-cdk-lib/aws-ecr';

interface PipelineStackProps extends cdk.StackProps {
    fargateService: FargateService;
    deploymentGroup: EcsDeploymentGroup;
    repository: Repository;
    connectionArn: string;
}

export class PipelineStack extends cdk.Stack {
    constructor(scope: Construct, id: string, props: PipelineStackProps) {
        super(scope, id, props);

        const sourceOutput = new Artifact();
        const buildOutput = new Artifact();

        const pipeline = new Pipeline(this, 'Pipeline', {
            pipelineName: 'MyPipeline',
            stages: [
                {
                    stageName: 'Source',
                    actions: [
                        new CodeStarConnectionsSourceAction({
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
                        new CodeBuildAction({
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
                        new CodeDeployEcsDeployAction({
                            actionName: 'ECS_Deploy',
                            deploymentGroup: props.deploymentGroup,
                            appSpecTemplateFile: buildOutput.atPath('appspec.yaml'),
                            taskDefinitionTemplateFile: buildOutput.atPath('taskdef.json'), //TODO generate this file from FargateTaskDefinition toString
                            containerImageInputs: [{input: buildOutput}]
                        }),
                    ],
                },
            ],
        });

        new cdk.CfnOutput(this, 'PipelineName', {
            value: pipeline.pipelineName,
        });
    }
}
