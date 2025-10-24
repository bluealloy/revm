package main

/*
#include <stdint.h>
*/
import "C"
import (
	"fmt"
	"math/big"
	"unsafe"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
)

// Error codes
const (
	OK                   = 0
	ERR_INVALID_G1_POINT = -1
	ERR_INVALID_G2_POINT = -2
	ERR_PAIRING_FAILED   = -3
	ERR_POINT_NOT_ON_CURVE = -4
)

//export gnark_bn254_g1_add
func gnark_bn254_g1_add(p1_bytes *C.uint8_t, p2_bytes *C.uint8_t, out *C.uint8_t) C.int {
	p1_slice := unsafe.Slice((*byte)(unsafe.Pointer(p1_bytes)), 64)
	p2_slice := unsafe.Slice((*byte)(unsafe.Pointer(p2_bytes)), 64)

	var p1, p2 bn254.G1Affine
	if err := decodeG1Point(p1_slice, &p1); err != nil {
		return ERR_INVALID_G1_POINT
	}
	if err := decodeG1Point(p2_slice, &p2); err != nil {
		return ERR_INVALID_G1_POINT
	}

	var result bn254.G1Affine
	result.Add(&p1, &p2)

	out_slice := unsafe.Slice((*byte)(unsafe.Pointer(out)), 64)
	encodeG1Point(&result, out_slice)
	return OK
}

//export gnark_bn254_g1_mul
func gnark_bn254_g1_mul(point_bytes *C.uint8_t, scalar_bytes *C.uint8_t, out *C.uint8_t) C.int {
	point_slice := unsafe.Slice((*byte)(unsafe.Pointer(point_bytes)), 64)
	scalar_slice := unsafe.Slice((*byte)(unsafe.Pointer(scalar_bytes)), 32)

	var point bn254.G1Affine
	if err := decodeG1Point(point_slice, &point); err != nil {
		return ERR_INVALID_G1_POINT
	}

	// Parse scalar as big-endian
	scalar := new(big.Int).SetBytes(scalar_slice)

	var result bn254.G1Affine
	result.ScalarMultiplication(&point, scalar)

	out_slice := unsafe.Slice((*byte)(unsafe.Pointer(out)), 64)
	encodeG1Point(&result, out_slice)
	return OK
}

//export gnark_bn254_pairing_check
func gnark_bn254_pairing_check(pairs_data *C.uint8_t, num_pairs C.int, result *C.uint8_t) C.int {
	pairSize := 192 // 64 (G1) + 128 (G2)
	data := unsafe.Slice((*byte)(unsafe.Pointer(pairs_data)), int(num_pairs)*pairSize)

	g1Points := make([]bn254.G1Affine, num_pairs)
	g2Points := make([]bn254.G2Affine, num_pairs)

	for i := 0; i < int(num_pairs); i++ {
		offset := i * pairSize
		g1_bytes := data[offset : offset+64]
		g2_bytes := data[offset+64 : offset+192]

		if err := decodeG1Point(g1_bytes, &g1Points[i]); err != nil {
			return ERR_INVALID_G1_POINT
		}
		if err := decodeG2Point(g2_bytes, &g2Points[i]); err != nil {
			return ERR_INVALID_G2_POINT
		}
	}

	ok, err := bn254.PairingCheck(g1Points, g2Points)
	if err != nil {
		return ERR_PAIRING_FAILED
	}

	result_ptr := (*byte)(unsafe.Pointer(result))
	if ok {
		*result_ptr = 1
	} else {
		*result_ptr = 0
	}

	return OK
}

// Helper functions
func decodeG1Point(bytes []byte, point *bn254.G1Affine) error {
	// Big-endian encoding: x (32 bytes) | y (32 bytes)
	var x, y fp.Element
	x.SetBytes(bytes[0:32])
	y.SetBytes(bytes[32:64])

	// Handle point at infinity
	if x.IsZero() && y.IsZero() {
		point.X.SetZero()
		point.Y.SetZero()
		return nil
	}

	point.X = x
	point.Y = y

	if !point.IsOnCurve() {
		return fmt.Errorf("point not on curve")
	}
	if !point.IsInSubGroup() {
		return fmt.Errorf("point not in subgroup")
	}

	return nil
}

func decodeG2Point(bytes []byte, point *bn254.G2Affine) error {
	// G2 encoding: x_imag | x_real | y_imag | y_real (32 bytes each)
	var x0, x1, y0, y1 fp.Element
	x1.SetBytes(bytes[0:32])   // imaginary part
	x0.SetBytes(bytes[32:64])  // real part
	y1.SetBytes(bytes[64:96])
	y0.SetBytes(bytes[96:128])

	// Handle point at infinity
	if x0.IsZero() && x1.IsZero() && y0.IsZero() && y1.IsZero() {
		point.X.SetZero()
		point.Y.SetZero()
		return nil
	}

	point.X.A0 = x0
	point.X.A1 = x1
	point.Y.A0 = y0
	point.Y.A1 = y1

	if !point.IsOnCurve() {
		return fmt.Errorf("point not on curve")
	}
	if !point.IsInSubGroup() {
		return fmt.Errorf("point not in subgroup")
	}

	return nil
}

func encodeG1Point(point *bn254.G1Affine, out []byte) {
	xBytes := point.X.Bytes()
	yBytes := point.Y.Bytes()
	copy(out[0:32], xBytes[:])
	copy(out[32:64], yBytes[:])
}

func main() {}
