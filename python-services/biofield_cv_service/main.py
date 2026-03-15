"""Biofield CV sidecar service.

FastAPI application on port 8002. Implements spatial algorithms from
biofield_spatial_algorithms.json.
"""

import uvicorn
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from biofield_cv_service.health import router as health_router
from biofield_cv_service.analyze import router as analyze_router

app = FastAPI(
    title="Selemene Biofield CV Service",
    version="3.0.0",
    description="Python sidecar service for biofield image analysis using OpenCV.",
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
    """Entry point for the `biofield-cv-service` console script."""
    uvicorn.run(
        "biofield_cv_service.main:app",
        host="0.0.0.0",
        port=8002,
        reload=True,
    )


if __name__ == "__main__":
    start()
