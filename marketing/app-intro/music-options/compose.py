"""Four original, gentle instrumental sketches for the 34-second promo."""
from pathlib import Path
import wave
import numpy as np

ROOT = Path(__file__).resolve().parent
SR, LENGTH = 48000, 34
N = SR * LENGTH
TAU = 2 * np.pi
rng = np.random.default_rng(26023)

def freq(note):
    return 440 * 2 ** ((note - 69) / 12)

def voice(note, duration, kind):
    t = np.arange(round(SR * duration)) / SR
    f = freq(note)
    release = np.minimum(1, (duration - t) / .45)
    if kind == 'piano':
        signal = np.zeros(len(t))
        for h, amp in [(1,1),(2,.22),(3,.11),(4,.045),(5,.018)]:
            signal += amp * np.sin(TAU*f*h*(1+.00012*h*h)*t) * np.exp(-t*(.85+h*.27))
        return signal * np.minimum(1,t/.012) * release
    if kind == 'pluck':
        signal = np.zeros(len(t))
        for h in range(1,9):
            signal += np.sin(TAU*f*h*t + h*.16) * (1/h**1.65) * np.exp(-t*(1.5+h*.45))
        return signal * np.minimum(1,t/.008) * release
    if kind == 'bell':
        signal = (np.sin(TAU*f*t)*np.exp(-t*1.2)
                  + .13*np.sin(TAU*f*2*t)*np.exp(-t*2.4)
                  + .035*np.sin(TAU*f*3.01*t)*np.exp(-t*4))
        return signal * np.minimum(1,t/.018) * release
    # Soft, slowly moving pad with a rounded attack and no bright transients.
    signal = (np.sin(TAU*f*t) + .36*np.sin(TAU*f*1.0012*t+.4)
              + .09*np.sin(TAU*f*2*t))
    attack = np.sin(np.minimum(1,t/1.3)*np.pi/2)**2
    return signal * attack * np.minimum(1,(duration-t)/1.6) * (.9+.1*np.sin(TAU*.17*t))

def compose(name, bpm, chords, melody, kind, pad_gain, lead_gain):
    audio = np.zeros((N,2),dtype=np.float64)
    beat = 60/bpm
    def place(note, when, duration, timbre, gain, pan=0):
        if when >= LENGTH:
            return
        sound = voice(note,duration,timbre)
        offset = round(when*SR)
        count = min(len(sound),N-offset)
        gains = np.array([np.sqrt((1-pan)/2),np.sqrt((1+pan)/2)])
        audio[offset:offset+count] += sound[:count,None]*gain*gains
    # Four phrases of eight beats; the final chord settles at 30 seconds.
    phrase_length = 7.5
    for phrase,chord in enumerate(chords):
        at = phrase*phrase_length
        for j,note in enumerate(chord):
            place(note,at,9,'pad',pad_gain,(j-1.5)*.25)
        place(chord[0]-12,at+.08,5.5,'piano',.06)
        if kind == 'piano':
            for j,b in enumerate([0,1.5,3,4.5,6,7.5]):
                place(chord[j%len(chord)]+12,at+b*beat,3.5,'piano',.065,(-1)**j*.22)
        elif kind == 'pluck':
            for j,b in enumerate([0,1,2.5,4,5,6.5,8]):
                place(chord[j%len(chord)]+12,at+b*beat,2.7,'pluck',.10,(-1)**j*.28)
                if j in (0,3):
                    for k,note in enumerate(chord):
                        place(note,at+b*beat+k*.035,2.5,'pluck',.055,(k-1.5)*.25)
        elif kind == 'bell':
            for j,b in enumerate([.5,2.5,4.5,6.5]):
                place(chord[j%len(chord)]+12,at+b*beat,4,'piano',.055,(-1)**j*.3)
        for j,(b,note) in enumerate(melody[phrase]):
            at_note=at+b*beat
            place(note,at_note,4.6 if kind!='pluck' else 3.1,kind,lead_gain,(-1)**j*.13)
            # Sparse, quiet stereo reflections rather than a busy rhythmic delay.
            place(note,at_note+.29,3,kind,lead_gain*.12,(-1)**(j+1)*.55)
    for j,note in enumerate(chords[0]):
        place(note,30+j*.045,4,'pad',pad_gain*.9,(j-1.5)*.2)
    place(chords[0][1]+12,30.2,3.8,kind,lead_gain*.7)
    # Low-level room reflections, with a fresh buffer to avoid feedback buildup.
    dry=audio.copy()
    for delay,gain in [(.071,.08),(.113,.06),(.193,.035)]:
        shift=round(SR*delay)
        audio[shift:] += dry[:-shift,::-1]*gain
    t=np.arange(N)/SR
    audio *= (np.minimum(1,t/.35)*np.minimum(1,(LENGTH-t)/2.2))[:,None]
    rms=np.sqrt(np.mean(audio**2))
    audio *= min(.085/max(rms,1e-9), .68/max(np.max(np.abs(audio)),1e-9))
    with wave.open(str(ROOT/f'{name}.wav'),'wb') as out:
        out.setparams((2,2,SR,0,'NONE','not compressed'))
        out.writeframes((np.clip(audio,-1,1)*32767).astype('<i2').tobytes())
    print(name, '34s', 'peak',round(float(np.max(np.abs(audio))),3),flush=True)

compose('01-cloud-piano',72,
 [(50,57,61,66),(47,54,57,62),(43,50,54,59),(45,52,57,61)],
 [[(.5,74),(2,78),(4,76),(6,73)],[(1,74),(3,73),(5.5,69)],
  [(.5,71),(2.5,74),(4.5,78),(6.5,76)],[(1,73),(3,69),(5.5,74)]],
 'piano',.033,.19)
compose('02-warm-strings',76,
 [(48,55,59,64),(45,52,55,60),(41,48,52,57),(43,50,55,59)],
 [[(1,72),(2.5,76),(4,79),(6,76)],[(.5,72),(2.5,71),(5,69),(7,67)],
  [(1,69),(3,72),(5,76)],[(.5,74),(2.5,71),(5,67),(7,72)]],
 'pluck',.024,.18)
compose('03-floating-light',64,
 [(41,48,52,57),(48,55,59,64),(50,57,60,65),(46,53,57,62)],
 [[(1.5,69),(4.5,72)],[(1,76),(4,71),(6.5,67)],
  [(2,69),(5,65)],[(1,65),(3.5,69),(6,72)]],
 'piano',.052,.11)
compose('04-starlight-box',68,
 [(43,50,54,59),(40,47,50,55),(48,55,59,64),(50,57,60,66)],
 [[(1,79),(2.5,78),(4,74),(6.5,71)],[(1,74),(3,78),(5,79)],
  [(.5,76),(2.5,79),(5,83),(7,79)],[(1,78),(3,74),(5.5,71)]],
 'bell',.031,.14)
