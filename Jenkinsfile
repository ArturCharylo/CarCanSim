pipeline {
    agent any 

    options {
        timestamps()
        disableConcurrentBuilds() 
    }

    stages {
        stage('Test & Lint (Rust)') {
            steps {
                sh 'docker build --target builder -t telemetry-builder .'
                sh 'docker run --rm telemetry-builder cargo test'
            }
        }

        stage('Build Image') {
            steps {
                sh 'docker build -t telemetry-app:latest .'
            }
        }

        stage('Deploy to Kubernetes') {
            steps {
                sh 'kubectl apply -f k8s/'
                sh 'kubectl rollout restart deployment telemetry-app'
            }
        }
    }

    post {
        always {
            sh 'docker image prune -f'
        }
        success {
            echo "Pipeline executed successfully! New version deployed to Kubernetes."
        }
        failure {
            echo "Pipeline failed. Check the Jenkins console output for troubleshooting."
        }
    }
}