# 規格書：Mahony 6-DOF 姿態解算四元數步進 (Mahony AHRS Step)

## 模組名稱
`sensor_mahony_ahrs`

## 業務目標與適用場景
在無人機、微型飛行器與穿戴動作捕捉設備中，Mahony 互補濾波演算法以低運算量、高數值穩定性聞名，能融合陀螺儀角速度與加速度計重力向量更新姿態四元數。
本模組提供零外部依賴、純 `std` 實作的 Mahony 6-DOF 單步解算微積木。

## 09 號合約邊界規範

### 函式 1：Mahony 6-DOF 四元數濾波單步 (Mahony 6-DOF IMU Step)
- **函式簽名**：
  ```rust
  /// @outputs: 4
  /// @domain: dt [0.0001..1.0]
  /// @domain: kp [0.0..100.0]
  /// @domain: ki [0.0..100.0]
  pub fn mahony_ahrs_update_6dof(
      q: &[f64; 4],
      gyro_rps: (f64, f64, f64),
      accel_g: (f64, f64, f64),
      integral_err: &mut (f64, f64, f64),
      kp: f64,
      ki: f64,
      dt: f64,
  ) -> Option<[f64; 4]>
  ```
- **演算法核心步驟**：
  - 邊界檢查：`dt <= 0.0 || kp < 0.0 || ki < 0.0` 則返回 `None`。
  - 正規化加速度計向量 $[a_x, a_y, a_z]$。若範數小於 $10^{-6}$，跳過重力修正，僅做陀螺儀純積分。
  - 估算重力向量方向：
    $v_x = 2(q_1 q_3 - q_0 q_2)$
    $v_y = 2(q_0 q_1 + q_2 q_3)$
    $v_z = q_0^2 - q_1^2 - q_2^2 + q_3^2$
  - 計算叉積誤差向量：
    $e_x = a_y v_z - a_z v_y$
    $e_y = a_z v_x - a_x v_z$
    $e_z = a_x v_y - a_y v_x$
  - 誤差積分與飽和抗飽和：
    $i_x += e_x \cdot ki \cdot dt$ ... 夾止於 $[-1.0, 1.0]$。
  - 修正角速度：
    $\omega_x = g_x + kp \cdot e_x + i_x$
  - 四元數導數積分：
    $\dot{q}_0 = 0.5 (-q_1 \omega_x - q_2 \omega_y - q_3 \omega_z)$
    $\dot{q}_1 = 0.5 (q_0 \omega_x + q_2 \omega_z - q_3 \omega_y)$
    $\dot{q}_2 = 0.5 (q_0 \omega_y - q_1 \omega_z + q_3 \omega_x)$
    $\dot{q}_3 = 0.5 (q_0 \omega_z + q_1 \omega_y - q_2 \omega_x)$
  - 步進更新並重新正規化四元數為單位向量。

## 黃金測試向量 (Golden Vectors)
- 初始狀態：單位四元數 $[1.0, 0.0, 0.0, 0.0]$，靜止於重力場中 $accel = (0.0, 0.0, 1.0)$，$gyro = (0.0, 0.0, 0.0)$：
  - 估計重力 $v = (0.0, 0.0, 1.0)$，誤差 $e = (0.0, 0.0, 0.0)$。
  - 輸出四元數保持 $[1.0, 0.0, 0.0, 0.0]$。
  - 嚴格遵守「物理恆等性與基準單位響應守則」。
- 容差使用 `(val - golden).abs() < 1e-4`。
