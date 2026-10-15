struct TextData {
  float4x4 transform;
  Glyph *glyphs;
  Curve *curves;
  // Texture2D<uint4>.Handle atlas;
  // Texture2D.Handle raw_atlas;
  uint atlas;
  uint sampler;
}

struct Glyph {
  float2 position;
  uint base_idx;
  uint n;
  uint2 base_offset;
  uint image_idx;
  uint layer;
  float2 min;
  float2 max;
  float2 cell_size;
}

struct Curve {
  float2 start;
  float2 control;
  float2 end;

  float eval(float t) {
    let one_sub_t = 1.0 - t;
    return start.y * one_sub_t * one_sub_t + 2.0 * control.y * t * one_sub_t +
           end.y * t * t;
  }

  void intesect_x(float value, float boundary, float k, inout Acc acc) {
    let denom = start.x - 2.0 * control.x + end.x;
    float root1;
    float root2;
    if (abs(denom) < 0.00001) {
      let root = (value - start.x) * 0.5 / (control.x - start.x);
      root1 = root;
      root2 = root;
    } else {
      let delta_l = control.x * control.x - start.x * end.x;
      let delta = delta_l + value * denom;
      if (delta < 0.0) {
        return;
      }
      let delta_sqrt = sqrt(delta);
      let rool_l = start.x - control.x;
      root1 = (rool_l - delta_sqrt) / denom;
      root2 = (rool_l + delta_sqrt) / denom;
    };

    let end_gt = end.x > value;
    let control_gt = control.x > value;
    let start_gt = start.x > value;

    if (start_gt && end_gt && !control_gt || !end_gt && start_gt) {
      let val = eval(root1);
      let offset = (k * -(val - boundary) + 0.5);
      if (offset >= 1.0) {
        acc.tot += 1;
        acc.base += 1;
      } else if (offset > 0.0) {
        acc.tot += 1;
        acc.count += 1;
        acc.offset += offset;
      }
    }

    if (end_gt && !start_gt || end_gt == start_gt && end_gt != control_gt) {
      let val = eval(root2);
      let offset = (k * -(val - boundary) + 0.5);
      if (offset >= 1.0) {
        acc.base -= 1;
      } else if (offset > 0.0) {
        acc.neg_count += 1;
        acc.neg_offset += offset;
      }
    }
  }

  int intesect_x_hard(float value, float boundary) {
    let denom = start.x - 2.0 * control.x + end.x;
    float root1;
    float root2;
    if (abs(denom) < 0.00001) {
      let root = (value - start.x) * 0.5 / (control.x - start.x);
      root1 = root;
      root2 = root;
    } else {
      let delta_l = control.x * control.x - start.x * end.x;
      let delta = delta_l + value * denom;
      if (delta < 0.0) {
        return 0;
      }
      let delta_sqrt = sqrt(delta);
      let rool_l = start.x - control.x;
      root1 = (rool_l - delta_sqrt) / denom;
      root2 = (rool_l + delta_sqrt) / denom;
    };

    let end_gt = end.x > value;
    let control_gt = control.x > value;
    let start_gt = start.x > value;

    var c = 0;

    if (start_gt && end_gt && !control_gt || !end_gt && start_gt) {
      let val = eval(root1);
      if (val <= boundary) {
        c += 1;
      }
    }

    if (end_gt && !start_gt || end_gt == start_gt && end_gt != control_gt) {
      let val = eval(root2);
      if (val <= boundary) {
        c -= 1;
      }
    }

    return c;
  }
}

struct Acc {
  float offset;
  int count;
  float neg_offset;
  int neg_count;
  int base;
  int tot;
}

struct TextVertex {
  float4 position : SV_Position;
  TextProps props : TextProps;
}

struct TextProps {
  float2 cell;
  uint2 base_offset;
  float2 cell_size;
  uint base_idx;
  uint n;
  uint layer;
  uint image_idx;
}

[shader("vertex")]
TextVertex textVert(uint vertex_id: SV_VertexID,
                    uint instance_id: SV_InstanceID, uniform TextData data) {
  let glyph = data.glyphs[instance_id];

  static const float d = 5.0;  

  float2 ns[] = { float2(glyph.n, 0), float2(glyph.n), float2(0, glyph.n),
                    float2(0, glyph.n), float2(0), float2(glyph.n, 0) };
  float2 offs[] = { float2(glyph.max.x, glyph.min.y),  glyph.max, float2(glyph.min.x, glyph.max.y),
                    float2(glyph.min.x, glyph.max.y), glyph.min, float2(glyph.max.x, glyph.min.y) };

  let vertex_off = offs[vertex_id] * float2(1.0, -1.0);
  let d_vertex_off = vertex_off + normalize(vertex_off) * d;

  TextProps props = { ns[vertex_id], glyph.base_offset, glyph.cell_size, glyph.base_idx, glyph.n, glyph.layer, glyph.image_idx };

  let p = float4((glyph.position + vertex_off), 0, 1);
  return { mul(data.transform, p), props };
}

[shader("fragment")]
float4 textFrag(uniform TextData data, TextProps props: TextProps) {
  let ideal = textFrag2(data, props);
  let offset = props.cell * props.cell_size;
  let w = 1.0/fwidth(offset);
  let uintcell = min(uint2((floor(props.cell))), props.n);
  Sampler2DArray.Handle raw_atlas = { uint2(props.image_idx, data.sampler) };
  Texture2D<uint4>.Handle atlas = { uint2(data.atlas, 0) };
  float elems = 0.0;
  float dwj = 0.0;
  float dhj = 0.0;
  float n_of_levels = 0.0;
  raw_atlas.GetDimensions(0, dwj, dhj, elems, n_of_levels);
  let raw = raw_atlas.SampleBias(float3(float2(props.cell / props.n), props.layer), 0.0);
  let range = atlas.Load(
      int3(props.base_offset + uintcell, 0));

  return float4(float3(1.0), raw.r);
}

float textFrag2(TextData data, TextProps props) {
  static const uint n = 5;

  float total = 0.0;

  // let center_cell =  * props.cell_size;
  let w = fwidth(props.cell);
  let corner_cell = props.cell - w/2.0;
  for (int x_off = 0; x_off < n; x_off += 1) {
    for (int y_off = 0; y_off < n; y_off += 1) {
      let cell = corner_cell + (float2(x_off, y_off)+float2(0.5)) * w/n;
      let uintcell = min(uint2((floor(cell))), props.n);
      Texture2D<uint4>.Handle atlas = { uint2(data.atlas, 0) };
      let range = atlas.Load(
          int3(props.base_offset + uintcell, 0));
      var value = float(int(range.r) - 128); // bias
      let offset = cell * props.cell_size;
      for (var i = range.g; i < range.b; i += 1) {
        let curve = data.curves[props.base_idx + i];
        value += curve.intesect_x_hard(offset.x, offset.y);
        let cell_start = floor(props.cell) * props.cell_size;
        let cell_end = cell_start + props.cell_size;

        if (curve.start.x >= cell_start.x && curve.start.x < cell_end.x &&
            curve.start.y < cell_start.y && offset.x < curve.start.x) {
          value -= 1;
        }

        if (curve.end.x >= cell_start.x && curve.end.x < cell_end.x &&
            curve.end.y < cell_start.y && offset.x < curve.end.x) {
          value += 1;
        }
      }
      total += float(value!=0)/(n*n);
    }
  }

  // let w = 1.0/fwidth(offset);

  return total;
}
