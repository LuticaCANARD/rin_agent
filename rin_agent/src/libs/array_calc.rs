#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// 두 f32 배열을 더합니다. (dst = a + b)
/// SIMD를 사용하여 최적화합니다.
pub fn add_arrays(a: &[f32], b: &[f32], dst: &mut [f32]) {
    let mut is_done = false;
    let len = a.len().min(b.len()).min(dst.len());
    let mut i = 0;

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        if !is_done {
            unsafe {
                while i + 8 <= len {
                    let va = _mm256_loadu_ps(a.as_ptr().add(i));
                    let vb = _mm256_loadu_ps(b.as_ptr().add(i));
                    let res = _mm256_add_ps(va, vb);
                    _mm256_storeu_ps(dst.as_mut_ptr().add(i), res);
                    i += 8;
                }
            }
            is_done = true;
        }
    }
    #[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
    {
        if !is_done {
                unsafe {
                    while i + 4 <= len {
                        let va = _mm_loadu_ps(a.as_ptr().add(i));
                        let vb = _mm_loadu_ps(b.as_ptr().add(i));
                    let res = _mm_add_ps(va, vb);
                    _mm_storeu_ps(dst.as_mut_ptr().add(i), res);
                    i += 4;
                }
            }
        }    
        is_done = true;
    }

    #[cfg(target_arch = "aarch64")]
    {
        if !is_done {
            unsafe {
                while i + 4 <= len {
                    let va = vld1q_f32(a.as_ptr().add(i));
                    let vb = vld1q_f32(b.as_ptr().add(i));
                    let res = vaddq_f32(va, vb);
                    vst1q_f32(dst.as_mut_ptr().add(i), res);
                    i += 4;
                }
            }
            is_done = true;
        }
    }

    // 남은 요소 처리 (Fallback)
    while i < len {
        dst[i] = a[i] + b[i];
        i += 1;
    }
}

/// 두 f32 배열의 내적(Dot Product)을 계산합니다.
/// SIMD를 사용하여 최적화합니다.
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    let mut is_done = false;
    let len = a.len().min(b.len());
    let mut i = 0;
    let mut sum = 0.0;

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        unsafe {
            let mut vsum = _mm256_setzero_ps();
            while i + 8 <= len {
                let va = _mm256_loadu_ps(a.as_ptr().add(i));
                let vb = _mm256_loadu_ps(b.as_ptr().add(i));
                // FMA (Fused Multiply-Add)가 지원되면 사용, 아니면 mul + add
                #[cfg(target_feature = "fma")]
                {
                    vsum = _mm256_fmadd_ps(va, vb, vsum);
                }
                #[cfg(not(target_feature = "fma"))]
                {
                    vsum = _mm256_add_ps(vsum, _mm256_mul_ps(va, vb));
                }
                i += 8;
            }
            
            // 수평 합산 (Horizontal Sum)
            // __m256 -> [f0, f1, f2, f3, f4, f5, f6, f7]
            // 1. 상위 128비트를 하위 128비트에 더함
            let vlow = _mm256_castps256_ps128(vsum);
            let vhigh = _mm256_extractf128_ps(vsum, 1);
            let vsum128 = _mm_add_ps(vlow, vhigh);
            
            // 2. SSE 수평 합산
            let mut temp = [0.0f32; 4];
            _mm_storeu_ps(temp.as_mut_ptr(), vsum128);
            sum += temp.iter().sum::<f32>();
            is_done = true;
        }
    }
    #[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
    {
        if !is_done {
            unsafe {
                let mut vsum = _mm_setzero_ps();
                while i + 4 <= len {
                    let va = _mm_loadu_ps(a.as_ptr().add(i));
                    let vb = _mm_loadu_ps(b.as_ptr().add(i));
                    // FMA (Fused Multiply-Add)가 지원되면 사용, 아니면 mul + add
                    #[cfg(target_feature = "fma")]
                    {
                        vsum = _mm_fmadd_ps(va, vb, vsum);
                    }
                    #[cfg(not(target_feature = "fma"))]
                    {
                        vsum = _mm_add_ps(vsum, _mm_mul_ps(va, vb));
                    }
                    i += 4;
                }
                // 수평 합산
                let mut temp = [0.0f32; 4];
                _mm_storeu_ps(temp.as_mut_ptr(), vsum);
                sum += temp.iter().sum::<f32>();
            }
        is_done = true;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if !is_done {
            unsafe {
                let mut vsum = vdupq_n_f32(0.0);
                while i + 4 <= len {
                    let va = vld1q_f32(a.as_ptr().add(i));
                    let vb = vld1q_f32(b.as_ptr().add(i));
                    vsum = vfmaq_f32(vsum, va, vb); // Fused Multiply-Add
                    i += 4;
                }
                // 수평 합산
                sum += vaddvq_f32(vsum);
            }
        is_done = true;
        }
    }

    // 남은 요소 처리
    while i < len {
        sum += a[i] * b[i];
        i += 1;
    }

    sum
}

/// 배열의 모든 요소에 상수를 곱합니다. (Scaling)
pub fn scale_array(a: &mut [f32], scale: f32) {
    let mut is_done = false;
    let len = a.len();
    let mut i = 0;

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        unsafe {
            let vscale = _mm256_set1_ps(scale);
            while i + 8 <= len {
                let va = _mm256_loadu_ps(a.as_ptr().add(i));
                let res = _mm256_mul_ps(va, vscale);
                _mm256_storeu_ps(a.as_mut_ptr().add(i), res);
                i += 8;
            }
        }
        is_done = true;
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
    {
        if !is_done {
            unsafe {
                let vscale = _mm_set1_ps(scale);
                while i + 4 <= len {
                    let va = _mm_loadu_ps(a.as_ptr().add(i));
                    let res = _mm_mul_ps(va, vscale);
                    _mm_storeu_ps(a.as_mut_ptr().add(i), res);
                    i += 4;
                }
            }
            is_done = true;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if !is_done {
            unsafe {
                let vscale = vdupq_n_f32(scale);
                while i + 4 <= len {
                    let va = vld1q_f32(a.as_ptr().add(i));
                    let res = vmulq_f32(va, vscale);
                    vst1q_f32(a.as_mut_ptr().add(i), res);
                    i += 4;
                }
            }
            is_done = true;
        }
    }

    while i < len {
        a[i] *= scale;
        i += 1;
    }
    return;

}
