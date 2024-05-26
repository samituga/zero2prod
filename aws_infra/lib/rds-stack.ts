import * as cdk from 'aws-cdk-lib';
import { DatabaseInstance, DatabaseInstanceEngine, PostgresEngineVersion } from 'aws-cdk-lib/aws-rds';
import { Vpc } from 'aws-cdk-lib/aws-ec2';
import { Construct } from 'constructs';

interface RdsStackProps extends cdk.StackProps {
    vpc: Vpc;
}

export class RdsStack extends cdk.Stack {
    public readonly dbInstance: DatabaseInstance;

    constructor(scope: Construct, id: string, props: RdsStackProps) {
        super(scope, id, props);

        this.dbInstance = new DatabaseInstance(this, 'Zero2ProdDB', {
            engine: DatabaseInstanceEngine.postgres({
                version: PostgresEngineVersion.VER_16_2,
            }),
            vpc: props.vpc,
            allocatedStorage: 20,
            maxAllocatedStorage: 40,
            instanceType: new cdk.aws_ec2.InstanceType('t3.micro'),
            multiAz: true,
            publiclyAccessible: false,
        });
    }
}
