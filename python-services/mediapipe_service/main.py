"""MediaPipe Face Mesh sidecar service.

FastAPI application on port 8001. Called by the Rust face-reading engine via HTTP.
"""

import uvicorn
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from mediapipe_service.health import router as health_router
from mediapipe_service.analyze import router as analyze_router

app = FastAPI(
    title="Selemene MediaPipe Face Mesh Service",
    version="3.0.0",
    description="Python sidecar service for MediaPipe Face Mesh landmark detection.",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(health_router)
app.include_router(analyze_router)


def start() -> None:
    """Entry point for the `mediapipe-service` console script."""
    uvicorn.run(
        "mediapipe_service.main:app",
        host="0.0.0.0",
        port=8001,
        reload=True,
    )


if __name__ == "__main__":
    start()
