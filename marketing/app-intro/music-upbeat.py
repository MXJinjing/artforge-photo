"""Original upbeat 120 BPM electro-pop cue, 34 seconds, D major."""
from pathlib import Path
import wave
import numpy as np

ROOT = Path(__file__).resolve().parent
SR, DURATION, BPM = 48000, 34, 120
audio = np.zeros((SR * DURATION, 2), dtype=np.float64)
rng = np.random.default_rng(703)

def hz(midi):
    return 440 * 2 ** ((midi - 69) / 12)

def add(signal, start, gain=1, pan=0):
    offset = round(start * SR)
    count = min(len(signal), len(audio) - offset)
    if offset < 0 or count <= 0:
        return
    gains = np.array([np.sqrt((1 - pan) / 2), np.sqrt((1 + pan) / 2)])
    audio[offset:offset + count] += signal[:count, None] * gains * gain

def tone(midi, dur, kind):
    t = np.arange(int(SR * dur)) / SR
    f = hz(midi)
    attack = np.minimum(1, t / .006)
    release = np.minimum(1, (dur - t) / .045)
    if kind == 'lead':
        s = np.sin(2*np.pi*f*t + 1.1*np.sin(2*np.pi*f*2*t)*np.exp(-t*12))
        s += .18*np.sin(2*np.pi*f*3*t)
        return s * attack * release * np.exp(-t*5)
    if kind == 'chord':
        s = np.sin(2*np.pi*f*t) + .3*np.sin(2*np.pi*f*2*t) + .12*np.sin(2*np.pi*f*3*t)
        return s * attack * release * np.exp(-t*8)
    s = np.sin(2*np.pi*f*t) + .22*np.sin(2*np.pi*f*2*t)
    return s * attack * release * np.exp(-t*3)

beat = 60 / BPM
chords = [(62,66,69,73), (69,73,76,78), (59,62,66,69), (55,59,62,66)]
melody = [
    [74,78,81,78,76,74,73,76],
    [76,81,85,81,78,76,73,76],
    [78,81,83,81,78,74,73,74],
    [79,83,81,78,76,74,73,69],
]
for bar in range(16):
    start = bar * 4 * beat
    chord = chords[bar % 4]
    # Bouncy off-beat chord stabs.
    for b in [.5, 1.5, 2.5, 3.5]:
        for j, note in enumerate(chord):
            add(tone(note, .33, 'chord'), start+b*beat+j*.008, .075, (j-1.5)*.24)
    for b, note in [(0,chord[0]-24),(1.5,chord[0]-12),(2,chord[0]-24),(3,chord[0]-17)]:
        add(tone(note, .34, 'bass'), start+b*beat, .26)
    # Bright plucked hook, plus a quiet stereo echo.
    for j, note in enumerate(melody[bar % 4]):
        gain = .13 if bar < 2 else .17
        if 10 <= bar <= 13:
            gain *= .78
        sound = tone(note, .44, 'lead')
        when = start+j*beat/2
        add(sound, when, gain, .12 if j%2 else -.12)
        add(sound, when+.1875, gain*.16, -.5 if j%2 else .5)
    for b in range(4):
        t = np.arange(int(SR*.24))/SR
        kick = np.sin(2*np.pi*(48*t + 6*(1-np.exp(-t*32))))*np.exp(-t*20)
        add(kick, start+b*beat, .42)
        if b in (1,3):
            t = np.arange(int(SR*.16))/SR
            noise = rng.standard_normal(len(t))
            noise = np.r_[0, np.diff(noise)]
            clap = noise * (np.exp(-t*36) + .5*np.exp(-np.maximum(t-.025,0)*60)*(t>.025))
            add(clap, start+b*beat, .075, .05)
    for eighth in range(8):
        t = np.arange(int(SR*.07))/SR
        noise = rng.standard_normal(len(t))
        hat = np.r_[0,np.diff(noise)]*np.exp(-t*70)*np.minimum(1,t/.001)
        add(hat, start+eighth*beat/2, .023 if eighth%2 else .016, -.35 if eighth%2 else .35)

# A final tonic resolves the cue at 32 seconds instead of cutting a loop short.
for j, note in enumerate((50,62,66,69,74)):
    add(tone(note, 1.9, 'lead'), 32+j*.025, .16, (j-2)*.2)
t = np.arange(len(audio))/SR
fade = np.minimum(1,t/.04)*np.minimum(1,(DURATION-t)/.65)
audio *= fade[:,None]
audio = np.tanh(audio*1.15)
audio *= .88/max(np.max(np.abs(audio)), .001)
with wave.open(str(ROOT/'music.wav'),'wb') as out:
    out.setparams((2,2,SR,0,'NONE','not compressed'))
    out.writeframes((audio*32767).astype('<i2').tobytes())
print(f'Original upbeat cue: {BPM} BPM, D major, {DURATION} seconds, peak {np.max(np.abs(audio)):.2f}')
