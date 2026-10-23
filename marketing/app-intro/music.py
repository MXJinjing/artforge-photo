"""Recreate the original gentle ambient cue, 34 seconds."""
from pathlib import Path
import wave
import numpy as np
ROOT = Path(__file__).resolve().parent
sr = 48000
t = np.arange(sr*34)/sr
chords = np.array([(146.832,220,293.665,369.994), (123.471,184.997,246.942,293.665),
                   (97.999,146.832,195.998,246.942), (110,164.814,220,277.183)])
freq = chords[(t/4.8).astype(int)%4]
phase = (t%4.8)/4.8
env = np.minimum(1,np.minimum(phase*9,(1-phase)*9))
pad = np.sum(np.sin(2*np.pi*freq*t[:,None])+.22*np.sin(2*np.pi*freq*1.002*t[:,None]),axis=1)*.035*env
step = (t/.6).astype(int)
age = t%.6
f = freq[np.arange(len(t)),step%4]*2
bell = (np.sin(2*np.pi*f*t)+.2*np.sin(2*np.pi*f*2*t))*np.exp(-age*7)*np.minimum(1,age*90)*.07
fade = np.minimum(1,np.minimum(t/1.5,(34-t)/2))
audio = np.column_stack(((pad+bell)*fade,(pad+bell*.85)*fade))
with wave.open(str(ROOT/'music.wav'),'wb') as out:
 out.setparams((2,2,sr,0,'NONE','not compressed'))
 out.writeframes((np.clip(audio,-1,1)*32767).astype('<i2').tobytes())
print('Restored original gentle ambient score.')
