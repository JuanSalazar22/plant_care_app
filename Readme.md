# Plant Care Web App (Rust Backend)

A simple web application built with Rust (Axum framework) to help you track watering and fertilizing schedules for your plants.

## Features

-   Add/Update/Remove plants.
-   Define watering and fertilizing frequency in days.
-   View all plants and their schedules.
-   See upcoming (next 3 days) and overdue watering/fertilizing tasks.
-   Mark plants as watered or fertilized (updates last action date).
-   Upload images for each plant to track growth visually.
-   View image history for a plant.
-   Data persisted in a local `data/plants.json` file.
-   Images stored locally in the `uploads/` directory.
-   Includes a `Dockerfile` for containerization.

## Acceptance Criteria Checklist

-   [x] Save information in a JSON file (`data/plants.json`)
-   [x] Read that file when started
-   [x] Allow adding a plant (name, water days, fertilize days)
-   [x] Allow seeing all plants and their scheduling info
-   [x] Allow checking the next 3 days and overdue tasks (with days overdue)
-   [x] Allow updating plant properties (name, water days, fertilize days)
-   [x] Allow removing a plant (also removes associated images)
-   [x] Allow uploading images to a plant
-   [x] Allow seeing the images of a plant (in details modal)
-   [x] Way to indicate plant was watered (button)
-   [x] Way to indicate plant was fertilized (button)
-   [x] Takes the system clock (`chrono::Utc::now().date_naive()`)
-   [x] `Dockerfile` for easy run
-   [x] When a plant is updated, their dates (last watered/fertilized) are **preserved**, not reset.
-   [x] Use `NaiveDate` from `chrono`.
-   [x] Create a README (this file).
-   [x] Instructions to deploy as a container in Google Cloud Run.

## Prerequisites

-   Rust (latest stable recommended): [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
-   Docker: [https://docs.docker.com/get-docker/](https://docs.docker.com/get-docker/)
-   Google Cloud SDK (`gcloud`): [https://cloud.google.com/sdk/docs/install](https://cloud.google.com/sdk/docs/install) (for Cloud Run deployment)
-   A Google Cloud Platform (GCP) project with Billing enabled and the Artifact Registry and Cloud Run APIs enabled.

## Running Locally

1.  **Clone the repository:**
    ```bash
    git clone <your-repo-url>
    cd plant_care_app
    ```
2.  **Create necessary directories:**
    ```bash
    mkdir data
    mkdir uploads
    mkdir static # If you didn't copy the HTML/CSS files yet
    # Copy index.html (and styles.css if used) into the static/ directory
    ```
3.  **Build and run the application:**
    ```bash
    cargo build --release
    ./target/release/plant_care_app
    ```
    Alternatively, for development:
    ```bash
    cargo run
    ```
4.  **Access the application:** Open your web browser and navigate to `http://localhost:3000`.

## Running with Docker

1.  **Build the Docker image:**
    ```bash
    docker build -t plant-care-app .
    ```
2.  **Run the Docker container:**
    ```bash
    docker run -p 3000:3000 \
           -v "$(pwd)/data":/app/data \
           -v "$(pwd)/uploads":/app/uploads \
           --name plant-care-container \
           plant-care-app
    ```
    *   `-p 3000:3000`: Maps port 3000 on your host to port 3000 in the container.
    *   `-v "$(pwd)/data":/app/data`: Mounts the local `data` directory into the container's `/app/data` directory. This persists your `plants.json` file outside the container. **Crucial for data persistence.**
    *   `-v "$(pwd)/uploads":/app/uploads`: Mounts the local `uploads` directory into the container for image persistence. **Crucial for image persistence.**
    *   `--name plant-care-container`: Assigns a name to the container for easier management.
    *   `plant-care-app`: The name of the image you built.

3.  **Access the application:** Open `http://localhost:3000`.

4.  **To stop the container:** `docker stop plant-care-container`
5.  **To remove the container:** `docker rm plant-care-container` (your data/uploads remain in the local mounted directories).

## Deploying to Google Cloud Run

**Important Note on Storage:** Google Cloud Run instances are ephemeral by default. This means the local filesystem (including the `data/plants.json` file and `uploads/` directory inside the container) will be **lost** whenever a new instance starts or replaces an old one (e.g., during scaling, updates, or crashes).

For **persistent** data and images in a production or serious environment on Cloud Run, you **must** use external storage:
*   **Data (JSON):** Use Google Cloud Storage (GCS) to read/write the JSON file, or better yet, switch to a database service like Cloud SQL or Firestore.
*   **Images:** Use Google Cloud Storage (GCS) buckets to store and serve images.

The instructions below deploy the app *without* persistent storage, suitable for **testing or demo purposes only**. Data and images will be lost on instance restarts.

---

1.  **Authenticate gcloud:**
    ```bash
    gcloud auth login
    gcloud config set project YOUR_GCP_PROJECT_ID
    ```

2.  **Enable APIs:**
    ```bash
    gcloud services enable run.googleapis.com
    gcloud services enable artifactregistry.googleapis.com
    ```

3.  **Create an Artifact Registry Repository:** (Choose a region close to you)
    ```bash
    gcloud artifacts repositories create plant-care-repo \
        --repository-format=docker \
        --location=us-central1 \ # Example region
        --description="Docker repository for Plant Care App"
    ```

4.  **Configure Docker to authenticate with Artifact Registry:**
    ```bash
    gcloud auth configure-docker us-central1-docker.pkg.dev # Use the same region
    ```

5.  **Build the Docker Image:** (Same as before)
    ```bash
    docker build -t plant-care-app .
    ```

6.  **Tag the Image for Artifact Registry:**
    Replace `YOUR_GCP_PROJECT_ID` and the region (`us-central1`) if you used a different one.
    ```bash
    docker tag plant-care-app us-central1-docker.pkg.dev/YOUR_GCP_PROJECT_ID/plant-care-repo/plant-care-app:latest
    ```

7.  **Push the Image to Artifact Registry:**
    ```bash
    docker push us-central1-docker.pkg.dev/YOUR_GCP_PROJECT_ID/plant-care-repo/plant-care-app:latest
    ```

8.  **Deploy to Cloud Run:**
    *   Replace `YOUR_GCP_PROJECT_ID` and region (`us-central1`).
    *   `--allow-unauthenticated`: Makes the service publicly accessible. Remove this for private services.
    *   `--port=3000`: Tells Cloud Run the container listens on port 3000.
    *   `--region=us-central1`: Deploy to this region.
    *   `--platform=managed`: Use the fully managed Cloud Run environment.

    ```bash
    gcloud run deploy plant-care-service \
        --image=us-central1-docker.pkg.dev/YOUR_GCP_PROJECT_ID/plant-care-repo/plant-care-app:latest \
        --port=3000 \
        --platform=managed \
        --region=us-central1 \
        --allow-unauthenticated \
        --set-env-vars=RUST_LOG=info # Optional: Set log level
        # Add --memory and --cpu flags if needed
    ```

9.  **Access the Deployed Service:** `gcloud` will output the Service URL after deployment.

**Remember the storage limitation mentioned above when using Cloud Run with this file-based storage approach.** For persistence, integrate with GCS or a database.