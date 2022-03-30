
[bpmodel.ContourStack]
id=BPC2

Points=POINT|1D|?x2|F32
Contours=RANGE|1D|?x2|U32
Attributes=INDEX|1D|?|U32
Metadata=JSON|1D|?|CUSTOM
PartIndex=INDEX|1D|?|U32
Slices=RANGE|1D|?x2|U32
Heights=NONE|1D|?|F32

Contours->Points
Attributes~Contours
Attributes->Metadata
PartIndex~Contours
Slices->Contours
Heights~Slices

[bpmodel.VectorStack]
id=BPV2

Vectors=HATCH|1D|?x4|F32
Blocks=RANGE|1D|?x2|U32
Attributes=INDEX|1D|?|U32
Metadata=JSON|1D|?|CUSTOM
PartIndex=INDEX|1D|?|U32
Slices=RANGE|1D|?x2|U32
Heights=NONE|1D|?|F32

Blocks->Vectors
Attributes~Blocks
Attributes->Metadata
PartIndex~Blocks
Slices->Blocks
Heights~Slices
