import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { CfnConnection } from 'aws-cdk-lib/aws-codestarconnections';

export class CodeStarStack extends cdk.Stack {
    public readonly connectionArn: string;

    constructor(scope: Construct, id: string, props?: cdk.StackProps) {
        super(scope, id, props);

        const connection = new CfnConnection(this, 'GitHubConnection', {
            connectionName: 'MyGitHubConnection',
            providerType: 'GitHub',
        });

        this.connectionArn = connection.attrConnectionArn;

        new cdk.CfnOutput(this, 'GitHubConnectionArn', {
            value: this.connectionArn,
        });
    }
}
