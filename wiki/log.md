## Development log

### 里程碑

| 日期        | 功能实现                                                 |
| --------- | ---------------------------------------------------- |
| 2025.3.25 | 基础 `compositor` 开发，实现 `client` 与 `compositor` 的简单通信。 |
| 2025.3.29 | 输入设备交互逻辑实现完成。                                        |
| 2025.4.16 | 优化布局算法，使用 `slotmap` 实现高效动态平铺。                        |

### 2025.3

阅读 `wayland` 协议相关内容，了解底层原理与通信逻辑，学习 `xdg-shell` 核心协议的实现。

阅读 `smithay` 源码与最小实现。

### 2025.3.25

项目正式启动，基于 `smithay-smallvil` 进行基础 `compositor` 的开发，实现了 `client` 与 `compositor` 的简单通信。

### 2025.3.26

实现 `输入设备交互-鼠标与键盘` 与 `compositor` 的交互。

### 2025.3.29

添加 `cursor` 的渲染，实现桌面管理器内部的鼠标图标渲染，支持 `XCursor-default` 与 `wl_surface`。

### 2025.3.31

修复一些已知 `bug`，优化了代码结构，解耦 `winit` 与 `state` 的初始化函数代码。

手写简单的边框渲染 `shader`，配合 `keyboard-focus` 实现了渲染当前焦点所在窗口的边框提示。

### 2025.4.2

添加 `WorkspaceManager` 与 `OutputManager`，实现了简易的工作区切换逻辑。

### 2025.4.3

整理代码结构，优化函数逻辑

### 2025.4.7

实现简易的平铺逻辑，新增 `render` 管理所有渲染请求，优化代码结构。

### 2025.4.8

为 `cursor` 的 `grab` 行为新增 `grab` 指针图标。

### 2025.4.10

实现基于 `平衡二叉树` 平铺排序算法，暂时未实现删除的逻辑。

### 2025.4.12

尝试实现 `layer-shell` 协议。

### 2025.4.13

实现 `layer-shell` 协议，能够支持 `waybar` 的使用，修改了 `input` 模块的代码结构，更加清晰，减少代码重复。

### 2025.4.14

实现 `viewporter` 协议，能够支持 `swww` 的使用，提供更换背景图片的功能。

实现了 `default-layout` 的布局算法，在当前 `focus` 的窗口下插入新窗口。

暂时还未实现删除的算法逻辑。

### 2025.4.16

测试发现删除速度过慢。改用 `slotmap` 建立 `平衡二叉树`，实现查找，插入，删除为 `O(1)` 级别的布局算法。实现动态平铺插入与删除。暂时未支持窗口切换与移动。

### 2025.4.17-25

完成部分文本撰写工作

完成窗口倒置操作

修改了output改变时布局未改变的bug

### 2025.4.26

实现了窗口的动态 resize

修改了多 surface 软件导致焦点丢失的问题 - 绑定根 surface 到 focus

### 2025.4.27 - 5.5

实现tty裸机模式下的图形化界面启动，实现drm设备的最小启动

TODO: 完善drm设备管理，完成渲染额外元素

### 2025.5.6-10

重构代码结构，新增GlobalData作为最大数据集合

TODO: 完善渲染部分代码

### 2025.5.11-12

优化代码结构，寻找实现shader着色器渲染的方法

### 2025.5.13-14

优化tty渲染逻辑，由统一的时钟发起帧渲染事件

### 2025.5.15-17

模仿 niri render的宏定义，实现shader着色器的渲染

### 2025.5.18-20

修复致命 shader 渲染死循环bug，实现 tty 模式下的项目启动

### 2025.5.21-5.23

优化 workspace manager 对于 windows 的管理，优化布局设置过程的管理。
引入 tiled 与 floating 两种 windows 布局情况，为后期平铺式堆叠式多支持做基础。
TODO：完善 resize 部分的代码

### 2025.5.24

完善平铺模式下的grab机制

### 2025.5.25-28

完善平铺模式下的resize机制，修复了 wl_surface 与 wl_subsurface 导致的冲突，grab时需设置 wl_surface 为 focus。

### 2025.5.29-30

添加平铺模式与浮动模式的切换