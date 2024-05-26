import * as cdk from 'aws-cdk-lib';
import {Artifact, Pipeline} from 'aws-cdk-lib/aws-codepipeline';
import {EcsDeployAction, GitHubSourceAction} from 'aws-cdk-lib/aws-codepipeline-actions';
import {BuildSpec, LinuxBuildImage, PipelineProject} from 'aws-cdk-lib/aws-codebuild';
import {Repository} from 'aws-cdk-lib/aws-ecr';
import {Cluster, FargateService} from 'aws-cdk-lib/aws-ecs';
import {Vpc} from 'aws-cdk-lib/aws-ec2';
import {Construct} from 'constructs';

interface PipelineStackProps extends cdk.StackProps {
    repository: Repository;
    ecsService: FargateService;
    ecsCluster: Cluster;
    vpc: Vpc;
}

export class PipelineStack extends cdk.Stack {
    constructor(scope: Construct, id: string, props: PipelineStackProps) {
        super(scope, id, props);

        const sourceOutput = new Artifact();
        const buildOutput = new Artifact();

        const pipeline = new Pipeline(this, 'Pipeline', {
            stages: [
                {
                    stageName: 'Source',
                    actions: [
                        new GitHubSourceAction({
                            actionName: 'GitHub',
                            owner: 'your-github-username',
                            repo: 'your-repo-name',
                            oauthToken: cdk.SecretValue.secretsManager('GITHUB_TOKEN'),
                            output: sourceOutput,
                            branch: 'main',
                        }),
                    ],
                },
                {
                    stageName: 'Build',
                    actions: [
                        new cdk.aws_codepipeline_actions.CodeBuildAction({
                            actionName: 'Build',
                            project: new PipelineProject(this, 'BuildProject', {
                                buildSpec: BuildSpec.fromObject({
                                    version: '0.2',
                                    phases: {
                                        install: {
                                            commands: ['npm install -g aws-cdk'],
                                        },
                                        build: {
                                            commands: [
                                                'cdk synth',
                                                'docker build -t my-rust-app .',
                                                `$(aws ecr get-login --no-include-email --region ${this.region})`,
                                                `docker tag my-rust-app:latest ${props.repository.repositoryUri}:latest`,
                                                `docker push ${props.repository.repositoryUri}:latest`,
                                            ],
                                        },
                                    },
                                    artifacts: {
                                        files: '**/*',
                                        'base-directory': 'cdk.out',
                                    },
                                }),
                                environment: {
                                    buildImage: LinuxBuildImage.STANDARD_5_0,
                                },
                            }),
                            input: sourceOutput,
                            outputs: [buildOutput],
                        }),
                    ],
                },
                {
                    stageName: 'Deploy',
                    actions: [
                        new EcsDeployAction({
                            actionName: 'Deploy',
                            service: props.ecsService,
                            input: buildOutput,
                        }),
                    ],
                },
            ],
        });
    }
}
