#!/usr/bin/env python3
"""
Generate mel filter banks for Whisper audio processing.
This script creates the binary mel filter files needed by the Rust implementation.
"""

import numpy as np
import struct

def mel_filter_bank(
    num_filters=80,
    num_fft=400,
    sample_rate=16000,
    num_mels=80,
    fmin=0.0,
    fmax=None,
):
    """Create a mel filter bank."""
    if fmax is None:
        fmax = float(sample_rate) / 2

    # Convert to mel scale
    def hz_to_mel(hz):
        return 2595.0 * np.log10(1.0 + hz / 700.0)

    def mel_to_hz(mel):
        return 700.0 * (10.0 ** (mel / 2595.0) - 1.0)

    # Create mel frequency points
    mel_points = np.linspace(hz_to_mel(fmin), hz_to_mel(fmax), num_filters + 2)
    
    # Convert back to Hz
    hz_points = mel_to_hz(mel_points)
    
    # Create FFT bin points
    bin_points = np.floor(hz_points * (num_fft // 2 + 1) / sample_rate).astype(int)
    
    # Create filter bank
    filters = np.zeros((num_filters, num_fft // 2 + 1))
    
    for i in range(1, num_filters + 1):
        # Lower triangle
        left = bin_points[i - 1]
        center = bin_points[i]
        right = bin_points[i + 1]
        
        # Create triangle filter
        for j in range(left, center):
            filters[i - 1, j] = (j - left) / (center - left)
        for j in range(center, right):
            filters[i - 1, j] = (right - j) / (right - center)
    
    return filters

def main():
    # Generate 80 mel filters (standard for Whisper)
    filters_80 = mel_filter_bank(num_filters=80, num_mels=80)
    
    # Generate 128 mel filters (for larger models)
    filters_128 = mel_filter_bank(num_filters=128, num_mels=128)
    
    # Save as binary files (little-endian float32)
    with open('melfilters.bytes', 'wb') as f:
        for filter_bank in [filters_80]:
            for row in filter_bank:
                for val in row:
                    f.write(struct.pack('<f', val))
    
    with open('melfilters128.bytes', 'wb') as f:
        for filter_bank in [filters_128]:
            for row in filter_bank:
                for val in row:
                    f.write(struct.pack('<f', val))
    
    print(f"Generated mel filter banks:")
    print(f"  melfilters.bytes: {filters_80.shape}")
    print(f"  melfilters128.bytes: {filters_128.shape}")

if __name__ == "__main__":
    main()