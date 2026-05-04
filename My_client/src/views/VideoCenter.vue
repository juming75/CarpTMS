<template>
  <div class="video-monitor-container">
    <!-- 顶部工具栏 -->
    <div class="monitor-toolbar">
      <!-- 左侧：设备统计 -->
      <div class="toolbar-left">
        <div class="device-stats">
          <el-tag type="info" size="small">全部 {{ totalDevices }}</el-tag>
          <el-tag type="success" size="small">在线 {{ onlineDevices }}</el-tag>
          <el-tag type="info" size="small">离线 {{ offlineDevices }}</el-tag>
          <el-tag type="warning" size="small">关注 {{ starredDevices }}</el-tag>
        </div>
      </div>

      <!-- 中间：分屏布局 -->
      <div class="toolbar-center">
        <div class="layout-buttons">
          <!-- 自定义分屏图标按钮 -->
          <el-tooltip content="1路" placement="bottom">
            <button class="layout-btn" :class="{ active: layout === 1 }" @click="setLayout(1)">
              <svg viewBox="0 0 20 20" width="20" height="20">
                <rect x="1" y="1" width="18" height="18" rx="2" fill="none" stroke="currentColor" stroke-width="1.5"/>
              </svg>
            </button>
          </el-tooltip>
          <el-tooltip content="4路（2×2）" placement="bottom">
            <button class="layout-btn" :class="{ active: layout === 4 }" @click="setLayout(4)">
              <svg viewBox="0 0 20 20" width="20" height="20">
                <rect x="1" y="1" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="11" y="1" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="1" y="11" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="11" y="11" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              </svg>
            </button>
          </el-tooltip>
          <el-tooltip content="6路（2×3）" placement="bottom">
            <button class="layout-btn" :class="{ active: layout === 6 }" @click="setLayout(6)">
              <svg viewBox="0 0 20 20" width="20" height="20">
                <rect x="1" y="1" width="5.3" height="8.5" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="7.3" y="1" width="5.3" height="8.5" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="13.6" y="1" width="5.3" height="8.5" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="1" y="10.5" width="5.3" height="8.5" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="7.3" y="10.5" width="5.3" height="8.5" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="13.6" y="10.5" width="5.3" height="8.5" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              </svg>
            </button>
          </el-tooltip>
          <el-tooltip content="9路（3×3）" placement="bottom">
            <button class="layout-btn" :class="{ active: layout === 9 }" @click="setLayout(9)">
              <svg viewBox="0 0 20 20" width="20" height="20">
                <rect x="1" y="1" width="5.3" height="5.3" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="7.3" y="1" width="5.3" height="5.3" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="13.6" y="1" width="5.3" height="5.3" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="1" y="7.3" width="5.3" height="5.3" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="7.3" y="7.3" width="5.3" height="5.3" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="13.6" y="7.3" width="5.3" height="5.3" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="1" y="13.6" width="5.3" height="5.3" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="7.3" y="13.6" width="5.3" height="5.3" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
                <rect x="13.6" y="13.6" width="5.3" height="5.3" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              </svg>
            </button>
          </el-tooltip>
          <el-tooltip content="16路（4×4）" placement="bottom">
            <button class="layout-btn" :class="{ active: layout === 16 }" @click="setLayout(16)">
              <svg viewBox="0 0 20 20" width="20" height="20">
                <rect x="1" y="1" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="5.7" y="1" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="10.3" y="1" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="15" y="1" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="1" y="5.7" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="5.7" y="5.7" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="10.3" y="5.7" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="15" y="5.7" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="1" y="10.3" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="5.7" y="10.3" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="10.3" y="10.3" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="15" y="10.3" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="1" y="15" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="5.7" y="15" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="10.3" y="15" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <rect x="15" y="15" width="4" height="4" rx="0.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
              </svg>
            </button>
          </el-tooltip>
          <el-tooltip content="25路（5×5）" placement="bottom">
            <button class="layout-btn" :class="{ active: layout === 25 }" @click="setLayout(25)">
              <svg viewBox="0 0 20 20" width="20" height="20">
                <rect x="1" y="1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="4.7" y="1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="8.4" y="1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="12.1" y="1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="15.8" y="1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="1" y="4.7" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="4.7" y="4.7" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="8.4" y="4.7" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="12.1" y="4.7" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="15.8" y="4.7" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="1" y="8.4" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="4.7" y="8.4" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="8.4" y="8.4" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="12.1" y="8.4" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="15.8" y="8.4" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="1" y="12.1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="4.7" y="12.1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="8.4" y="12.1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="12.1" y="12.1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="15.8" y="12.1" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="1" y="15.8" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="4.7" y="15.8" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="8.4" y="15.8" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="12.1" y="15.8" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
                <rect x="15.8" y="15.8" width="3.2" height="3.2" rx="0.3" fill="none" stroke="currentColor" stroke-width="1"/>
              </svg>
            </button>
          </el-tooltip>
          <el-tooltip content="36路（6×6）" placement="bottom">
            <button class="layout-btn" :class="{ active: layout === 36 }" @click="setLayout(36)">
              <svg viewBox="0 0 20 20" width="20" height="20">
                <rect x="1" y="1" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="4" y="1" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="7" y="1" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="10" y="1" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="13" y="1" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="16" y="1" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="1" y="4" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="4" y="4" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="7" y="4" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="10" y="4" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="13" y="4" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="16" y="4" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="1" y="7" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="4" y="7" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="7" y="7" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="10" y="7" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="13" y="7" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="16" y="7" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="1" y="10" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="4" y="10" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="7" y="10" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="10" y="10" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="13" y="10" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="16" y="10" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="1" y="13" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="4" y="13" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="7" y="13" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="10" y="13" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="13" y="13" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="16" y="13" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="1" y="16" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="4" y="16" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="7" y="16" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="10" y="16" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="13" y="16" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
                <rect x="16" y="16" width="2.5" height="2.5" rx="0.2" fill="none" stroke="currentColor" stroke-width="0.8"/>
              </svg>
            </button>
          </el-tooltip>
        </div>

        <!-- 功能按钮 -->
        <div class="function-buttons">
          <!-- 关闭/打开所有视频切换按钮 -->
          <el-tooltip :content="allStreamsClosed ? '打开所有视频' : '关闭所有视频'" placement="bottom">
            <button class="func-btn danger" :class="{ active: !allStreamsClosed }" @click="toggleAllStreams">
              <el-icon><Close v-if="!allStreamsClosed" /><VideoCamera v-else /></el-icon>
              {{ allStreamsClosed ? '打开' : '关闭' }}
            </button>
          </el-tooltip>
          <!-- 视频/音频/混合模式切换按钮 -->
          <el-tooltip :content="`流媒体模式: ${streamModeLabel}`" placement="bottom">
            <button class="func-btn mode-btn" @click="toggleStreamMode">
              <el-icon><VideoCamera v-if="streamMode === 'video'" /><Microphone v-else-if="streamMode === 'audio'" /><Connection v-else /></el-icon>
              {{ streamModeLabel }}
            </button>
          </el-tooltip>
          <el-tooltip content="语音对讲" placement="bottom">
            <button class="func-btn" :class="{ active: talkEnabled }" @click="toggleTalk">
              <el-icon><Microphone /></el-icon> 对讲
            </button>
          </el-tooltip>
          <el-tooltip content="轮巡播放" placement="bottom">
            <button class="func-btn" :class="{ active: tourEnabled }" @click="toggleTour">
              <el-icon><RefreshRight /></el-icon> 轮巡
            </button>
          </el-tooltip>
          <el-tooltip content="360度全景" placement="bottom">
            <button class="func-btn" @click="openPanorama">
              <el-icon><Aim /></el-icon> 360
            </button>
          </el-tooltip>
          <el-tooltip content="全屏显示" placement="bottom">
            <button class="func-btn" @click="toggleFullscreen">
              <el-icon><FullScreen /></el-icon> 全屏
            </button>
          </el-tooltip>
        </div>
      </div>

      <!-- 右侧：搜索和刷新 -->
      <div class="toolbar-right">
        <el-input
          v-model="searchText"
          placeholder="搜索车牌号/设备号/组织机构"
          clearable
          size="small"
          class="search-input"
          @keyup.enter="handleSearch"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-button size="small" :icon="Refresh" @click="refreshDevices" />
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="monitor-main">
      <!-- 左侧设备树 -->
      <div class="device-tree-panel" :class="{ collapsed: treeCollapsed }">
        <div class="tree-header">
          <span>设备列表</span>
          <el-icon class="collapse-btn" @click="treeCollapsed = !treeCollapsed">
            <DArrowLeft v-if="!treeCollapsed" />
            <DArrowRight v-else />
          </el-icon>
        </div>
        <el-tree
          ref="treeRef"
          :data="deviceTree"
          :props="treeProps"
          node-key="id"
          :expand-on-click-node="false"
          :highlight-current="true"
          @node-click="handleTreeNodeClick"
          class="custom-tree"
        >
          <template #default="{ node, data }">
            <span class="tree-node-content">
              <!-- 复选框 -->
              <el-checkbox
                v-if="data.type === 'device'"
                :model-value="selectedDevices.includes(data.id)"
                @change="(val: boolean) => toggleDeviceSelect(data, val)"
                @click.stop
              />
              <!-- 图标 -->
              <el-icon v-if="data.type === 'org'" :size="14" color="#909399"><OfficeBuilding /></el-icon>
              <el-icon v-else-if="data.type === 'group'" :size="14" color="#409eff"><FolderOpened /></el-icon>
              <el-icon v-else-if="data.status === 'online'" :size="14" color="#67c23a"><Van /></el-icon>
              <el-icon v-else :size="14" color="#909399"><Van /></el-icon>
              <!-- 名称 -->
              <span class="node-label" :title="node.label">{{ node.label }}</span>
              <!-- 状态标签 -->
              <span v-if="data.type === 'device'" class="node-status">
                <span class="status-dot" :class="data.status"></span>
                <span class="status-text">{{ data.status === 'online' ? '在线' : '离线' }}</span>
              </span>
              <span v-else-if="data.type === 'org' || data.type === 'group'" class="node-count">
                ({{ data.count || 0 }})
              </span>
            </span>
          </template>
        </el-tree>
      </div>

      <!-- 中间视频区域 -->
      <div class="video-area">
        <!-- 视频网格 -->
        <div class="video-grid" :class="`grid-${layout}`">
          <div
            v-for="i in layout"
            :key="i"
            class="video-cell"
            :class="{ 
              active: videoSlots[i-1]?.device,
              selected: selectedCell === i-1,
              hasAlarm: videoSlots[i-1]?.hasAlarm
            }"
            @click="selectCell(i-1)"
            @dblclick="toggleCellFullscreen(i-1)"
          >
            <!-- 有视频 -->
            <div v-if="videoSlots[i-1]?.device" class="cell-content">
              <div class="cell-header">
                <span class="cell-index">{{ i }}</span>
                <span class="cell-title">{{ videoSlots[i-1].device.name }}</span>
                <span v-if="videoSlots[i-1].channel" class="cell-channel">CH{{ videoSlots[i-1].channel }}</span>
                <div class="cell-actions">
                  <el-icon class="action-icon" @click.stop="captureSnapshot(i-1)" title="截图">
                    <Camera />
                  </el-icon>
                  <el-icon class="action-icon" @click.stop="toggleRecording(i-1)" :title="videoSlots[i-1].isRecording ? '停止录制' : '开始录制'">
                    <VideoCamera v-if="!videoSlots[i-1].isRecording" />
                    <VideoPause v-else />
                  </el-icon>
                  <el-icon class="action-icon" @click.stop="toggleAudio(i-1)" :title="videoSlots[i-1].muted ? '开启声音' : '静音'">
                    <Microphone v-if="!videoSlots[i-1].muted" />
                    <Mute v-else />
                  </el-icon>
                  <el-icon class="action-icon" @click.stop="removeFromCell(i-1)" title="移除">
                    <Close />
                  </el-icon>
                </div>
              </div>
              <div class="cell-player">
                <canvas
                  v-if="videoSlots[i-1].streamActive"
                  :ref="el => setCanvasRef(i-1, el)"
                  class="player-canvas"
                ></canvas>
                <div v-else class="player-placeholder">
                  <svg viewBox="0 0 80 80" width="60" height="60" class="channel-logo">
                    <circle cx="40" cy="40" r="35" fill="none" stroke="rgba(255,255,255,0.6)" stroke-width="2"/>
                    <path d="M30 25 L55 40 L30 55 Z" fill="rgba(255,255,255,0.6)"/>
                    <circle cx="40" cy="40" r="28" fill="none" stroke="rgba(255,255,255,0.3)" stroke-width="1"/>
                  </svg>
                  <span v-if="videoSlots[i-1].connecting" class="connecting-text">连接中...</span>
                  <span v-else-if="videoSlots[i-1].signalLost" class="error-text">信号丢失</span>
                  <span v-else class="waiting-text">等待视频流</span>
                </div>
                <!-- 录制指示 -->
                <div v-if="videoSlots[i-1].isRecording" class="recording-badge">
                  <span class="rec-dot"></span>
                  REC
                </div>
                <!-- 告警指示 -->
                <div v-if="videoSlots[i-1].hasAlarm" class="alarm-badge">
                  <el-icon><Warning /></el-icon>
                  {{ videoSlots[i-1].alarmCount }}
                </div>
              </div>
              <div class="cell-footer">
                <span class="cell-time">{{ currentTime }}</span>
                <span v-if="videoSlots[i-1].fps" class="cell-stats">{{ videoSlots[i-1].fps }}FPS</span>
              </div>
            </div>
            <!-- 空槽位 -->
            <div v-else class="cell-empty" @click="showCellPicker(i-1)">
              <svg viewBox="0 0 40 40" width="32" height="32" class="empty-icon">
                <rect x="4" y="4" width="32" height="32" rx="4" fill="none" stroke="#555" stroke-width="2" stroke-dasharray="4,4"/>
                <line x1="20" y1="14" x2="20" y2="26" stroke="#555" stroke-width="2"/>
                <line x1="14" y1="20" x2="26" y2="20" stroke="#555" stroke-width="2"/>
              </svg>
              <span>双击或点击添加视频</span>
            </div>
          </div>
        </div>

        <!-- 底部标签栏 -->
        <div class="bottom-tabs">
          <el-tabs v-model="activeTab" class="monitor-tabs">
            <el-tab-pane label="车辆监控" name="monitor">
              <el-table :data="monitoredVehicles" stripe size="small" class="monitor-table" height="180">
                <el-table-column type="index" label="序号" width="50" />
                <el-table-column prop="plateNumber" label="车牌号" width="100" />
                <el-table-column prop="status" label="车辆状态" width="80">
                  <template #default="{ row }">
                    <el-tag :type="row.status === '行驶' ? 'success' : row.status === '离线' ? 'info' : 'warning'" size="small">
                      {{ row.status }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="commTime" label="通信时间" width="160" />
                <el-table-column prop="locTime" label="定位时间" width="160" />
                <el-table-column prop="locStatus" label="定位状态" width="80">
                  <template #default="{ row }">
                    <el-tag :type="row.locStatus === '已定位' ? 'success' : 'danger'" size="small">
                      {{ row.locStatus }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="deviceNo" label="设备号" width="120" />
                <el-table-column prop="orgName" label="组织机构" min-width="100" />
                <el-table-column label="操作" width="150">
                  <template #default="{ row }">
                    <el-button link type="primary" size="small" @click="playVehicle(row)">视频</el-button>
                    <el-button link type="primary" size="small" @click="locateVehicle(row)">定位</el-button>
                    <el-button link type="primary" size="small" @click="sendCommand(row)">指令</el-button>
                  </template>
                </el-table-column>
              </el-table>
            </el-tab-pane>
            <el-tab-pane label="报警信息" name="alarm">
              <el-table :data="alarmList" stripe size="small" class="monitor-table" height="180">
                <el-table-column type="index" label="序号" width="50" />
                <el-table-column prop="time" label="报警时间" width="160" />
                <el-table-column prop="plateNumber" label="车牌号" width="100" />
                <el-table-column prop="alarmType" label="报警类型" width="120">
                  <template #default="{ row }">
                    <el-tag :type="getAlarmTypeColor(row.alarmType)" size="small">{{ row.alarmType }}</el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="content" label="报警内容" min-width="200" />
                <el-table-column prop="status" label="处理状态" width="80">
                  <template #default="{ row }">
                    <el-tag :type="row.status === '已处理' ? 'success' : 'danger'" size="small">{{ row.status }}</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </el-tab-pane>
            <el-tab-pane label="媒体文件" name="media">
              <div class="media-panel">
                <el-empty description="暂无媒体文件" />
              </div>
            </el-tab-pane>
          </el-tabs>
        </div>
      </div>

      <!-- 右侧地图面板 -->
      <div class="map-panel" :class="{ collapsed: mapCollapsed }">
        <div class="map-header">
          <span>实时定位</span>
          <el-icon class="collapse-btn" @click="mapCollapsed = !mapCollapsed">
            <DArrowRight v-if="!mapCollapsed" />
            <DArrowLeft v-else />
          </el-icon>
        </div>
        <div class="map-container">
          <!-- 模拟地图 -->
          <div class="map-placeholder">
            <div class="map-bg">
              <div class="map-grid"></div>
            </div>
            <!-- 选中的车辆信息浮窗 -->
            <div v-if="selectedVehicleInfo" class="vehicle-info-card">
              <div class="card-header">
                <el-icon :size="24" color="#409eff"><Van /></el-icon>
                <span class="card-title">{{ selectedVehicleInfo.name }}</span>
                <el-tag size="small" :type="selectedVehicleInfo.status === 'online' ? 'success' : 'info'">
                  {{ selectedVehicleInfo.status === 'online' ? '在线' : '离线' }}
                </el-tag>
                <el-icon class="close-btn" @click="selectedVehicleInfo = null"><Close /></el-icon>
              </div>
              <div class="card-body">
                <div class="info-row">
                  <span class="info-label">从业人员</span>
                  <span class="info-value">{{ selectedVehicleInfo.driver || '未绑定' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">车辆状态</span>
                  <span class="info-value">{{ selectedVehicleInfo.vehicleStatus || '未知' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">通信信号</span>
                  <span class="info-value">{{ selectedVehicleInfo.signal || '未获取' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">通信时间</span>
                  <span class="info-value">{{ selectedVehicleInfo.commTime || '--' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">设备号</span>
                  <span class="info-value">{{ selectedVehicleInfo.deviceNo || '--' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">油量</span>
                  <span class="info-value">{{ selectedVehicleInfo.fuel || '--' }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">组织</span>
                  <span class="info-value">{{ selectedVehicleInfo.org || '--' }}</span>
                </div>
              </div>
              <div class="card-actions">
                <el-button size="small" type="primary" @click="playVehicle(selectedVehicleInfo)">视频</el-button>
                <el-button size="small" @click="sendCommand(selectedVehicleInfo)">指令</el-button>
                <el-button size="small" @click="viewHistory(selectedVehicleInfo)">轨迹</el-button>
              </div>
            </div>
          </div>
        </div>
        <!-- 地图底部工具栏 -->
        <div class="map-toolbar">
          <el-button-group size="small">
            <el-tooltip content="清空" placement="top">
              <el-button :icon="Delete" @click="clearMapMarkers" />
            </el-tooltip>
            <el-tooltip content="导出" placement="top">
              <el-button :icon="Download" @click="exportMapData" />
            </el-tooltip>
          </el-button-group>
          <el-button-group size="small">
            <el-tooltip content="消控" placement="top">
              <el-button :icon="Bell" />
            </el-tooltip>
            <el-tooltip content="导出" placement="top">
              <el-button :icon="Download" />
            </el-tooltip>
          </el-button-group>
          <el-icon class="setting-icon" @click="showMapSettings"><Setting /></el-icon>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import {
  Close, Microphone, Mute, Refresh, RefreshRight, Search,
  FullScreen, Aim, Van, FolderOpened, OfficeBuilding,
  DArrowLeft, DArrowRight, Camera, VideoCamera, VideoPause,
  Warning, Delete, Download, Bell, Setting, Connection
} from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';

// ====== 布局 ======
const layout = ref(4);
const activeTab = ref('monitor');
const selectedCell = ref<number | null>(null);
const currentTime = ref(new Date().toLocaleTimeString());

// ====== 面板控制 ======
const treeCollapsed = ref(false);
const mapCollapsed = ref(false);

// ====== 功能状态 ======
const tourEnabled = ref(false);
const talkEnabled = ref(false);

// ====== 搜索 ======
const searchText = ref('');

// ====== 设备树 ======
interface TreeNode {
  id: string;
  label: string;
  type: 'org' | 'group' | 'device';
  status?: 'online' | 'offline';
  count?: number;
  children?: TreeNode[];
  device?: any;
}

const deviceTree = ref<TreeNode[]>([
  {
    id: 'org-1',
    label: '视频主动安全',
    type: 'org',
    count: 32,
    children: [
      {
        id: 'grp-1',
        label: '测1215(离线)',
        type: 'group',
        count: 1,
        children: []
      },
      {
        id: 'grp-2',
        label: '测试9C(离线,报警)',
        type: 'group',
        count: 1,
        children: []
      },
      {
        id: 'grp-3',
        label: '粤A2L568(离线,报警)',
        type: 'group',
        count: 1,
        children: []
      },
      {
        id: 'grp-4',
        label: '粤A2L567(离线,报警)',
        type: 'group',
        count: 1,
        children: []
      },
      {
        id: 'grp-5',
        label: '粤N68917(离线,停车)',
        type: 'group',
        count: 1,
        children: []
      },
      {
        id: 'grp-6',
        label: '苏AF28648(停车)',
        type: 'group',
        count: 1,
        children: [
          { id: 'dev-1-ch1', label: 'CH1', type: 'device', status: 'online' as const, device: { id: 1, name: '苏AF28648', channel: 1, deviceNo: '595074203044', org: '视频主动安全', driver: '张三', vehicleStatus: '停车', signal: '良好', commTime: '2026-04-26 11:35:43', fuel: '0L', plateNumber: '苏AF28648' } },
          { id: 'dev-1-ch2', label: 'CH2', type: 'device', status: 'online' as const, device: { id: 1, name: '苏AF28648', channel: 2, deviceNo: '595074203044', org: '视频主动安全' } },
          { id: 'dev-1-ch3', label: 'CH3', type: 'device', status: 'online' as const, device: { id: 1, name: '苏AF28648', channel: 3, deviceNo: '595074203044', org: '视频主动安全' } },
          { id: 'dev-1-ch4', label: 'CH4', type: 'device', status: 'online' as const, device: { id: 1, name: '苏AF28648', channel: 4, deviceNo: '595074203044', org: '视频主动安全' } },
        ]
      },
    ]
  },
  {
    id: 'org-2',
    label: 'XX3895(离线)',
    type: 'org',
    count: 1,
    children: []
  },
  {
    id: 'org-3',
    label: '场内(粤H00000)(离线)',
    type: 'org',
    count: 1,
    children: []
  },
  {
    id: 'org-4',
    label: 'XXA12345(离线)',
    type: 'org',
    count: 1,
    children: []
  },
  {
    id: 'org-5',
    label: '粤A00000(离线)',
    type: 'org',
    count: 1,
    children: []
  },
  {
    id: 'org-6',
    label: '鄂A12345(离线)',
    type: 'org',
    count: 1,
    children: []
  },
  {
    id: 'org-7',
    label: '湘A68888(离线,报警)',
    type: 'org',
    count: 1,
    children: []
  },
  {
    id: 'org-8',
    label: '苏AF28615(离线)',
    type: 'org',
    count: 1,
    children: []
  },
  {
    id: 'org-9',
    label: '陕A66666(离线)',
    type: 'org',
    count: 1,
    children: []
  },
  {
    id: 'org-10',
    label: '鄂A5677GH(离线)',
    type: 'org',
    count: 1,
    children: []
  }
]);

const treeProps = {
  children: 'children',
  label: 'label'
};

// ====== 视频槽位 ======
interface VideoSlot {
  device: any | null;
  channel?: number;
  streamActive: boolean;
  connecting: boolean;
  signalLost: boolean;
  muted: boolean;
  fps: number;
  isRecording: boolean;
  hasAlarm: boolean;
  alarmCount: number;
  ws: WebSocket | null;
}

const videoSlots = ref<VideoSlot[]>(Array.from({ length: 36 }, () => createEmptySlot()));
const canvasRefs = ref<Map<number, HTMLCanvasElement>>(new Map());

function createEmptySlot(): VideoSlot {
  return {
    device: null,
    channel: undefined,
    streamActive: false,
    connecting: false,
    signalLost: false,
    muted: true,
    fps: 0,
    isRecording: false,
    hasAlarm: false,
    alarmCount: 0,
    ws: null,
  };
}

// ====== 设备选择 ======
const selectedDevices = ref<string[]>([]);

// ====== 选中车辆信息 ======
const selectedVehicleInfo = ref<any>(null);

// ====== 统计数据 ======
const totalDevices = computed(() => countDevices(deviceTree.value));
const onlineDevices = computed(() => countOnlineDevices(deviceTree.value));
const offlineDevices = computed(() => totalDevices.value - onlineDevices.value);
const starredDevices = ref(0);

function countDevices(nodes: TreeNode[]): number {
  let count = 0;
  for (const node of nodes) {
    if (node.type === 'device') count++;
    if (node.children) count += countDevices(node.children);
  }
  return count;
}

function countOnlineDevices(nodes: TreeNode[]): number {
  let count = 0;
  for (const node of nodes) {
    if (node.type === 'device' && node.status === 'online') count++;
    if (node.children) count += countOnlineDevices(node.children);
  }
  return count;
}

// ====== 车辆监控表格数据 ======
const monitoredVehicles = ref([
  { plateNumber: '595074203044', status: '行驶', commTime: '2026-04-26 11:35:43', locTime: '2026-04-26 09:05:41', locStatus: '已定位', deviceNo: '595074203044', orgName: '视频主动安全' },
  { plateNumber: '59079187300', status: '离线', commTime: '2026-04-26 03:27:19', locTime: '2026-04-09 23:26:50', locStatus: '已定位', deviceNo: '59079187300', orgName: '视频主动安全' },
  { plateNumber: '成功123', status: '离线', commTime: '2026-03-27 14:45:14', locTime: '2026-03-27 14:46:48', locStatus: '未定位', deviceNo: '18270774105', orgName: '视频主动安全' },
  { plateNumber: '测1215', status: '离线', commTime: '2026-01-14 23:01:57', locTime: '', locStatus: '未定位', deviceNo: '595073917844', orgName: '视频主动安全' },
  { plateNumber: '', status: '离线', commTime: '', locTime: '', locStatus: '', deviceNo: '24120400558', orgName: '' },
]);

// ====== 报警列表 ======
const alarmList = ref([
  { time: '2026-04-26 11:30:00', plateNumber: '粤A2L568', alarmType: '超速报警', content: '车辆当前速度85km/h，超过限速60km/h', status: '未处理' },
  { time: '2026-04-26 10:15:00', plateNumber: '粤A2L567', alarmType: '疲劳驾驶', content: '连续驾驶超过4小时未休息', status: '已处理' },
  { time: '2026-04-26 09:45:00', plateNumber: '苏AF28648', alarmType: '越界报警', content: '车辆驶出电子围栏区域', status: '未处理' },
]);

function getAlarmTypeColor(type: string) {
  if (type.includes('超速')) return 'danger';
  if (type.includes('疲劳')) return 'warning';
  if (type.includes('越界')) return 'warning';
  return 'info';
}

// ====== 布局切换 ======
function setLayout(count: number) {
  layout.value = count;
  // 重置槽位
  videoSlots.value = Array.from({ length: 36 }, () => createEmptySlot());
}

// ====== 功能切换 ======
function toggleTour() {
  tourEnabled.value = !tourEnabled.value;
  ElMessage.info(tourEnabled.value ? '轮巡已开启' : '轮巡已停止');
}

function toggleTalk() {
  talkEnabled.value = !talkEnabled.value;
  ElMessage.info(talkEnabled.value ? '对讲已开启' : '对讲已关闭');
}

// ====== 流媒体模式切换 ======
type StreamMode = 'video' | 'audio' | 'hybrid';
const streamMode = ref<StreamMode>('video');
const streamModeLabel = computed(() => {
  switch (streamMode.value) {
    case 'video': return '视频';
    case 'audio': return '音频';
    case 'hybrid': return '混合';
  }
});

function toggleStreamMode() {
  const modes: StreamMode[] = ['video', 'audio', 'hybrid'];
  const currentIndex = modes.indexOf(streamMode.value);
  streamMode.value = modes[(currentIndex + 1) % modes.length];
  ElMessage.success(`流媒体模式: ${streamModeLabel.value}`);
}

// ====== 关闭/打开所有视频切换 ======
const allStreamsClosed = ref(false);

function toggleAllStreams() {
  if (allStreamsClosed.value) {
    // 打开所有视频：恢复之前的视频槽位
    allStreamsClosed.value = false;
    ElMessage.success('已打开所有视频');
  } else {
    // 关闭所有视频
    allStreamsClosed.value = true;
    videoSlots.value.forEach(slot => {
      if (slot.ws) slot.ws.close();
    });
    videoSlots.value = Array.from({ length: 36 }, () => createEmptySlot());
    ElMessage.info('已关闭所有视频');
  }
}

function openPanorama() {
  ElMessage.info('360度全景功能');
}

function toggleFullscreen() {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen();
  } else {
    document.exitFullscreen();
  }
}

// @ts-expect-error reserved for closing all video streams
function _closeAllStreams() {
  videoSlots.value.forEach(slot => {
    if (slot.ws) slot.ws.close();
  });
  videoSlots.value = Array.from({ length: 36 }, () => createEmptySlot());
  ElMessage.success('已关闭所有视频');
}

function refreshDevices() {
  ElMessage.success('刷新设备列表');
}

function handleSearch() {
  ElMessage.info(`搜索: ${searchText.value}`);
}

// ====== 树操作 ======
function handleTreeNodeClick(data: TreeNode) {
  if (data.type === 'device' && data.device) {
    selectVehicleInfo(data.device);
  }
}

function toggleDeviceSelect(data: TreeNode, checked: boolean) {
  if (checked) {
    if (!selectedDevices.value.includes(data.id)) {
      selectedDevices.value.push(data.id);
    }
    // 自动播放到第一个空槽位
    const emptyIdx = videoSlots.value.findIndex(s => !s.device);
    if (emptyIdx >= 0 && data.device) {
      playToDevice(data.device, emptyIdx);
    }
  } else {
    const idx = selectedDevices.value.indexOf(data.id);
    if (idx >= 0) selectedDevices.value.splice(idx, 1);
    // 移除对应槽位
    const slotIdx = videoSlots.value.findIndex(s => s.device?.id === data.device?.id && s.channel === data.device?.channel);
    if (slotIdx >= 0) removeFromCell(slotIdx);
  }
}

// ====== 单元格操作 ======
function selectCell(index: number) {
  selectedCell.value = index;
  const slot = videoSlots.value[index];
  if (slot?.device) {
    selectVehicleInfo(slot.device);
  }
}

function toggleCellFullscreen(index: number) {
  const slot = videoSlots.value[index];
  if (slot?.device) {
    selectVehicleInfo(slot.device);
  }
}

function playToDevice(device: any, cellIndex: number) {
  const slot = videoSlots.value[cellIndex];
  slot.device = device;
  slot.channel = device.channel;
  slot.connecting = true;
  
  // 模拟连接
  setTimeout(() => {
    slot.connecting = false;
    slot.streamActive = true;
    slot.fps = 25;
  }, 1000);
  
  ElMessage.success(`已播放 ${device.name} CH${device.channel}`);
}

function showCellPicker(_index: number) {
  ElMessage.info('请从左侧设备树选择设备');
}

function removeFromCell(index: number) {
  const slot = videoSlots.value[index];
  if (slot.ws) slot.ws.close();
  videoSlots.value[index] = createEmptySlot();
  // 移除选中状态
  const selIdx = selectedDevices.value.findIndex(id => id.includes(String(slot.device?.id)));
  if (selIdx >= 0) selectedDevices.value.splice(selIdx, 1);
}

function captureSnapshot(index: number) {
  const canvas = canvasRefs.value.get(index);
  if (canvas) {
    const link = document.createElement('a');
    link.download = `snapshot_${Date.now()}.png`;
    link.href = canvas.toDataURL('image/png');
    link.click();
    ElMessage.success('截图已保存');
  } else {
    ElMessage.warning('当前无视频画面');
  }
}

function toggleRecording(index: number) {
  const slot = videoSlots.value[index];
  slot.isRecording = !slot.isRecording;
  ElMessage.success(slot.isRecording ? '开始录制' : '停止录制');
}

function toggleAudio(index: number) {
  const slot = videoSlots.value[index];
  slot.muted = !slot.muted;
  ElMessage.success(slot.muted ? '已静音' : '已开启声音');
}

function setCanvasRef(index: number, el: any) {
  if (el && el instanceof HTMLCanvasElement) {
    canvasRefs.value.set(index, el);
  }
}

// ====== 车辆信息 ======
function selectVehicleInfo(info: any) {
  selectedVehicleInfo.value = {
    name: info.name || info.plateNumber || '未知',
    status: info.status || 'online',
    driver: info.driver || '未绑定',
    vehicleStatus: info.vehicleStatus || '未知',
    signal: info.signal || '未获取',
    commTime: info.commTime || '--',
    deviceNo: info.deviceNo || '--',
    fuel: info.fuel || '--',
    org: info.org || '--',
    plateNumber: info.plateNumber || '',
  };
}

function playVehicle(info: any) {
  ElMessage.info(`播放 ${info.name || info.plateNumber} 视频`);
}

function locateVehicle(info: any) {
  selectVehicleInfo(info);
  ElMessage.info(`定位 ${info.name || info.plateNumber}`);
}

function sendCommand(info: any) {
  ElMessage.info(`发送指令到 ${info.name || info.plateNumber}`);
}

function viewHistory(info: any) {
  ElMessage.info(`查看 ${info.name || info.plateNumber} 历史轨迹`);
}

function clearMapMarkers() {
  ElMessage.success('已清空地图标记');
}

function exportMapData() {
  ElMessage.success('地图数据已导出');
}

function showMapSettings() {
  ElMessage.info('地图设置');
}

// ====== 定时刷新时间 ======
let timeInterval: number | null = null;

onMounted(() => {
  timeInterval = window.setInterval(() => {
    currentTime.value = new Date().toLocaleTimeString();
  }, 1000);
});

onUnmounted(() => {
  if (timeInterval) clearInterval(timeInterval);
  videoSlots.value.forEach(slot => {
    if (slot.ws) slot.ws.close();
  });
});
</script>

<style scoped>
/* ====== 容器 ====== */
.video-monitor-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1d23;
  color: #e0e0e0;
  font-family: 'Microsoft YaHei', sans-serif;
}

/* ====== 顶部工具栏 ====== */
.monitor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  background: #252830;
  border-bottom: 1px solid #3a3d45;
  min-height: 42px;
}

.toolbar-left { display: flex; align-items: center; }

.device-stats { display: flex; gap: 6px; }

.toolbar-center {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
  justify-content: center;
}

.layout-buttons {
  display: flex;
  gap: 4px;
  align-items: center;
}

.layout-btn {
  width: 32px;
  height: 28px;
  border: 1px solid #4a4d55;
  background: #2a2d35;
  color: #909399;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 3px;
  transition: all 0.2s;
}

.layout-btn:hover {
  background: #3a3d45;
  color: #e0e0e0;
}

.layout-btn.active {
  background: #409eff;
  border-color: #409eff;
  color: #fff;
}

.function-buttons {
  display: flex;
  gap: 4px;
  align-items: center;
}

.func-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: 1px solid #4a4d55;
  background: #2a2d35;
  color: #909399;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
}

.func-btn:hover {
  background: #3a3d45;
  color: #e0e0e0;
}

.func-btn.active {
  background: #67c23a;
  border-color: #67c23a;
  color: #fff;
}

.func-btn.danger:hover {
  background: #f56c6c;
  border-color: #f56c6c;
  color: #fff;
}

.func-btn.mode-btn {
  background: #409eff;
  border-color: #409eff;
  color: #fff;
}

.func-btn.mode-btn:hover {
  background: #66b1ff;
  border-color: #66b1ff;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-input {
  width: 220px;
}

.search-input :deep(.el-input__wrapper) {
  background: #2a2d35;
  border-color: #4a4d55;
}

.search-input :deep(.el-input__inner) {
  color: #e0e0e0;
}

/* ====== 主内容区 ====== */
.monitor-main {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* ====== 设备树面板 ====== */
.device-tree-panel {
  width: 240px;
  min-width: 200px;
  background: #1e2128;
  border-right: 1px solid #3a3d45;
  display: flex;
  flex-direction: column;
  transition: width 0.3s;
  overflow: hidden;
}

.device-tree-panel.collapsed {
  width: 0;
  min-width: 0;
  border: none;
}

.tree-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #252830;
  border-bottom: 1px solid #3a3d45;
  font-size: 13px;
  font-weight: 500;
}

.collapse-btn {
  cursor: pointer;
  color: #909399;
  transition: color 0.2s;
}

.collapse-btn:hover { color: #409eff; }

.custom-tree {
  flex: 1;
  overflow-y: auto;
  background: transparent;
  color: #e0e0e0;
}

.custom-tree :deep(.el-tree-node__content) {
  height: 28px;
  background: transparent;
}

.custom-tree :deep(.el-tree-node__content:hover) {
  background: #2a2d35;
}

.custom-tree :deep(.el-tree-node.is-current > .el-tree-node__content) {
  background: #1a3a5c;
}

.tree-node-content {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  font-size: 12px;
}

.node-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-status {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 11px;
  color: #909399;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status-dot.online { background: #67c23a; }
.status-dot.offline { background: #606266; }

.node-count {
  font-size: 11px;
  color: #606266;
}

/* ====== 视频区域 ====== */
.video-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ====== 视频网格 ====== */
.video-grid {
  flex: 1;
  display: grid;
  gap: 2px;
  padding: 2px;
  background: #0d0f14;
  overflow: hidden;
}

.grid-1 { grid-template-columns: 1fr; grid-template-rows: 1fr; }
.grid-4 { grid-template-columns: repeat(2, 1fr); grid-template-rows: repeat(2, 1fr); }
.grid-6 { grid-template-columns: repeat(3, 1fr); grid-template-rows: repeat(2, 1fr); }
.grid-9 { grid-template-columns: repeat(3, 1fr); grid-template-rows: repeat(3, 1fr); }
.grid-16 { grid-template-columns: repeat(4, 1fr); grid-template-rows: repeat(4, 1fr); }
.grid-25 { grid-template-columns: repeat(5, 1fr); grid-template-rows: repeat(5, 1fr); }
.grid-36 { grid-template-columns: repeat(6, 1fr); grid-template-rows: repeat(6, 1fr); }

.video-cell {
  background: #1a1d23;
  border: 1px solid #2a2d35;
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.video-cell.selected { border-color: #409eff; }
.video-cell.hasAlarm { border-color: #f56c6c; }
.video-cell.active { background: #0d0f14; }

.cell-content {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.cell-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.7);
  font-size: 11px;
  min-height: 24px;
}

.cell-index {
  color: #409eff;
  font-weight: bold;
  min-width: 16px;
}

.cell-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cell-channel {
  color: #909399;
  font-size: 10px;
}

.cell-actions {
  display: flex;
  gap: 4px;
}

.action-icon {
  cursor: pointer;
  color: #909399;
  font-size: 14px;
  transition: color 0.2s;
}

.action-icon:hover { color: #fff; }

.cell-player {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  background: #1489d4;
  overflow: hidden;
}

.player-canvas {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.player-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.channel-logo { opacity: 0.7; }
.connecting-text { color: rgba(255,255,255,0.8); font-size: 12px; }
.error-text { color: #f56c6c; font-size: 12px; }
.waiting-text { color: rgba(255,255,255,0.5); font-size: 12px; }

.recording-badge {
  position: absolute;
  top: 6px;
  right: 6px;
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(0, 0, 0, 0.6);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 10px;
  color: #f56c6c;
  font-weight: bold;
}

.rec-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #f56c6c;
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.alarm-badge {
  position: absolute;
  top: 6px;
  left: 6px;
  display: flex;
  align-items: center;
  gap: 3px;
  background: rgba(245, 108, 108, 0.8);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 10px;
  color: #fff;
}

.cell-footer {
  display: flex;
  justify-content: space-between;
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.7);
  font-size: 10px;
  color: #909399;
}

.cell-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: #555;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.cell-empty:hover {
  color: #409eff;
  background: rgba(64, 158, 255, 0.05);
}

.empty-icon { opacity: 0.5; }

/* ====== 底部标签 ====== */
.bottom-tabs {
  border-top: 1px solid #3a3d45;
  background: #1e2128;
}

.monitor-tabs :deep(.el-tabs__header) {
  margin: 0;
  background: #252830;
}

.monitor-tabs :deep(.el-tabs__item) {
  color: #909399;
  font-size: 13px;
}

.monitor-tabs :deep(.el-tabs__item.is-active) {
  color: #409eff;
}

.monitor-table {
  background: transparent;
}

.monitor-table :deep(.el-table) {
  background: #1e2128;
  color: #e0e0e0;
}

.monitor-table :deep(.el-table__row) {
  background: #1e2128;
}

.monitor-table :deep(.el-table__row:hover) {
  background: #252830;
}

.monitor-table :deep(th) {
  background: #252830 !important;
  color: #909399;
  font-weight: normal;
}

.media-panel {
  padding: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* ====== 地图面板 ====== */
.map-panel {
  width: 320px;
  min-width: 280px;
  background: #1e2128;
  border-left: 1px solid #3a3d45;
  display: flex;
  flex-direction: column;
  transition: width 0.3s;
  overflow: hidden;
}

.map-panel.collapsed {
  width: 0;
  min-width: 0;
  border: none;
}

.map-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #252830;
  border-bottom: 1px solid #3a3d45;
  font-size: 13px;
  font-weight: 500;
}

.map-container {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.map-placeholder {
  width: 100%;
  height: 100%;
  background: #1a3a5c;
  position: relative;
}

.map-bg {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #1a3a5c 0%, #0d2137 100%);
}

.map-grid {
  width: 100%;
  height: 100%;
  background-image: 
    linear-gradient(rgba(255,255,255,0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255,255,255,0.03) 1px, transparent 1px);
  background-size: 20px 20px;
}

/* 车辆信息卡片 */
.vehicle-info-card {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 240px;
  background: rgba(30, 33, 40, 0.95);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: #252830;
  position: relative;
}

.card-title {
  flex: 1;
  font-size: 14px;
  font-weight: 500;
}

.close-btn {
  cursor: pointer;
  color: #909399;
}

.close-btn:hover { color: #f56c6c; }

.card-body {
  padding: 10px 12px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  font-size: 12px;
  border-bottom: 1px solid #2a2d35;
}

.info-label { color: #909399; }
.info-value { color: #e0e0e0; }

.card-actions {
  display: flex;
  gap: 8px;
  padding: 10px 12px;
  border-top: 1px solid #2a2d35;
}

.card-actions .el-button { flex: 1; }

/* 地图工具栏 */
.map-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  background: #252830;
  border-top: 1px solid #3a3d45;
}

.setting-icon {
  cursor: pointer;
  color: #909399;
  font-size: 16px;
}

.setting-icon:hover { color: #409eff; }

/* ====== 滚动条 ====== */
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: #1a1d23; }
::-webkit-scrollbar-thumb { background: #3a3d45; border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: #4a4d55; }
</style>
