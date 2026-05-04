export class VideoFrameManager {
  private frameBuffers = new Map<number, HTMLCanvasElement>();
  private frameTimestamps = new Map<number, number>();
  private maxFramesPerVideo = 30;
  private cleanupInterval: number | null = null;

  constructor() {
    this.startCleanup();
  }

  getFrameBuffer(slotIndex: number): HTMLCanvasElement {
    let buffer = this.frameBuffers.get(slotIndex);
    if (!buffer) {
      buffer = document.createElement('canvas');
      this.frameBuffers.set(slotIndex, buffer);
    }
    return buffer;
  }

  updateFrame(slotIndex: number, frameData: ImageData | HTMLImageElement | string) {
    const buffer = this.getFrameBuffer(slotIndex);
    const ctx = buffer.getContext('2d');
    if (!ctx) return;

    if (typeof frameData === 'string') {
      const img = new Image();
      img.onload = () => {
        buffer.width = img.width;
        buffer.height = img.height;
        ctx.drawImage(img, 0, 0);
      };
      img.src = frameData;
    } else if (frameData instanceof ImageData) {
      buffer.width = frameData.width;
      buffer.height = frameData.height;
      ctx.putImageData(frameData, 0, 0);
    } else {
      buffer.width = frameData.width;
      buffer.height = frameData.height;
      ctx.drawImage(frameData, 0, 0);
    }

    this.frameTimestamps.set(slotIndex, Date.now());
  }

  releaseFrameBuffer(slotIndex: number) {
    const buffer = this.frameBuffers.get(slotIndex);
    if (buffer) {
      buffer.width = 0;
      buffer.height = 0;
      this.frameBuffers.delete(slotIndex);
      this.frameTimestamps.delete(slotIndex);
    }
  }

  releaseAllFrames() {
    this.frameBuffers.forEach((buffer) => {
      buffer.width = 0;
      buffer.height = 0;
    });
    this.frameBuffers.clear();
    this.frameTimestamps.clear();
  }

  private startCleanup() {
    this.cleanupInterval = window.setInterval(() => {
      const now = Date.now();
      this.frameTimestamps.forEach((timestamp, slotIndex) => {
        if (now - timestamp > 60000) {
          this.releaseFrameBuffer(slotIndex);
        }
      });
    }, 30000);
  }

  destroy() {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }
    this.releaseAllFrames();
  }

  getStats() {
    return {
      activeBuffers: this.frameBuffers.size,
      totalBytes: this.calculateMemoryUsage(),
    };
  }

  private calculateMemoryUsage(): number {
    let total = 0;
    this.frameBuffers.forEach((buffer) => {
      total += buffer.width * buffer.height * 4;
    });
    return total;
  }
}

export const videoFrameManager = new VideoFrameManager();