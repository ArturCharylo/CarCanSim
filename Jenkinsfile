pipeline {
    agent any

    options {
        timestamps()
        disableConcurrentBuilds() 
    }

    stages {
        stage('Test & Lint (Rust)') {
            steps {
                dir('/var/jenkins_home/workspace/telemetry-pipeline') {
                    // Build only the 'builder' stage which contains Rust, dependencies and the source code
                    sh 'docker build --target builder -t telemetry-builder .'
                    
                    sh 'docker run --rm telemetry-builder cargo test'
                }
            }
        }
        
        stage('Build Environment') {
            steps {
                dir('/var/jenkins_home/workspace/telemetry-pipeline') {
                    sh 'docker compose build'
                }
            }
        }
        
        stage('Deploy Locally') {
            steps {
                dir('/var/jenkins_home/workspace/telemetry-pipeline') {
                    sh 'docker compose up -d'
                }
            }
        }
    }

    post {
        always {
            // Prune dangling images to save local storage
            sh 'docker image prune -f'
        }
    }
}